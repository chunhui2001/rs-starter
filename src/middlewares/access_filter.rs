
use std::{
    borrow::Cow,
    collections::HashSet,
    convert::TryFrom,
    env,
    fmt::{self, Display as _},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_utils::future::{ready, Ready}; 
use bytes::Bytes;
use futures_core::ready; 
use log::{debug, warn};
use pin_project_lite::pin_project; 
use regex::{Regex, RegexSet};
use std::time::Duration;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use myhumantime::format_duration;
use human_repr::HumanCount;

use actix_web::body::{BodySize, MessageBody};
use actix_web::{ dev::{Service, ServiceRequest, ServiceResponse, Transform}, http::header::HeaderName, Error, Result }; 

#[derive(Debug)]
pub struct Logger(Rc<Inner>);

#[derive(Debug, Clone)]
struct Inner {
    format: Format,
    exclude: HashSet<String>,
    exclude_regex: RegexSet,
    log_target: Cow<'static, str>,
}

impl Logger {

    pub fn new(format: &str) -> Logger {
        Logger(Rc::new(Inner {
            format: Format::new(format),
            exclude: HashSet::new(),
            exclude_regex: RegexSet::empty(),
            log_target: Cow::Borrowed(module_path!()),
        }))
    }

    pub fn exclude<T: Into<String>>(mut self, path: T) -> Self {
        Rc::get_mut(&mut self.0)
            .unwrap()
            .exclude
            .insert(path.into());
        self
    }

    pub fn exclude_regex<T: Into<String>>(mut self, path: T) -> Self {
        let inner = Rc::get_mut(&mut self.0).unwrap();
        let mut patterns = inner.exclude_regex.patterns().to_vec();
        patterns.push(path.into());
        let regex_set = RegexSet::new(patterns).unwrap();
        inner.exclude_regex = regex_set;
        self
    }

    // pub fn log_target(mut self, target: impl Into<Cow<'static, str>>) -> Self {
    //     let inner = Rc::get_mut(&mut self.0).unwrap();
    //     inner.log_target = target.into();
    //     self
    // }

}

impl Default for Logger {
    fn default() -> Logger {
        Logger(Rc::new(Inner {
            format: Format::default(),
            exclude: HashSet::new(),
            exclude_regex: RegexSet::empty(),
            log_target: Cow::Borrowed(module_path!()),
        }))
    }
}

impl<S, B> Transform<S, ServiceRequest> for Logger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Response = ServiceResponse<StreamLog<B>>;
    type Error = Error;
    type Transform = LoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        for unit in &self.0.format.0 {
            if let FormatText::CustomRequest(label, None) = unit {
                warn!(
                    "No custom request replacement function was registered for label: {}",
                    label
                );
            }

            if let FormatText::CustomResponse(label, None) = unit {
                warn!(
                    "No custom response replacement function was registered for label: {}",
                    label
                );
            }
        }

        ready(Ok(LoggerMiddleware {
            service,
            inner: self.0.clone(),
        }))
    }
}

/// Logger middleware service.
pub struct LoggerMiddleware<S> {
    inner: Rc<Inner>,
    service: S,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Response = ServiceResponse<StreamLog<B>>;
    type Error = Error;
    type Future = LoggerResponse<S, B>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let excluded = self.inner.exclude.contains(req.path())
            || self.inner.exclude_regex.is_match(req.path());

        if excluded {
            LoggerResponse {
                fut: self.service.call(req),
                format: None,
                time: OffsetDateTime::now_utc(),
                log_target: Cow::Borrowed(""),
                _phantom: PhantomData,
            }
        } else {
            let now = OffsetDateTime::now_utc();
            let mut format = self.inner.format.clone();

            for unit in &mut format.0 {
                unit.render_request(now, &req);
            }

            LoggerResponse {
                fut: self.service.call(req),
                format: Some(format),
                time: now,
                log_target: self.inner.log_target.clone(),
                _phantom: PhantomData,
            }
        }
    }
}

pin_project! {
    pub struct LoggerResponse<S, B>
    where
        B: MessageBody,
        S: Service<ServiceRequest>,
    {
        #[pin]
        fut: S::Future,
        time: OffsetDateTime,
        format: Option<Format>,
        log_target: Cow<'static, str>,
        _phantom: PhantomData<B>,
    }
}

impl<S, B> Future for LoggerResponse<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<StreamLog<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let res = match ready!(this.fut.poll(cx)) {
            Ok(res) => res,
            Err(e) => return Poll::Ready(Err(e)),
        };

        if let Some(error) = res.response().error() {
            debug!("Error in response: {:?}", error);
        }

        let res = if let Some(ref mut format) = this.format {
            // to avoid polluting all the Logger types with the body parameter we swap the body
            // out temporarily since it's not usable in custom response functions anyway

            let (req, res) = res.into_parts();
            let (res, body) = res.into_parts();

            let temp_res = ServiceResponse::new(req, res.map_into_boxed_body());

            for unit in &mut format.0 {
                unit.render_response(&temp_res);
            }

            // re-construct original service response
            let (req, res) = temp_res.into_parts();
            ServiceResponse::new(req, res.set_body(body))
        } else {
            res
        };

        let time = *this.time;
        let format = this.format.take();
        let log_target = this.log_target.clone();
        let status_code = res.status().as_u16();

        Poll::Ready(Ok(res.map_body(move |_, body| StreamLog {
            body,
            time,
            format,
            size: 0,
            log_target,
            status_code,
        })))
    }
}

pin_project! {
    
    pub struct StreamLog<B> {
        #[pin]
        body: B,
        format: Option<Format>,
        size: usize,
        time: OffsetDateTime,
        log_target: Cow<'static, str>,
        status_code: u16,
    }

    impl<B> PinnedDrop for StreamLog<B> {
        fn drop(this: Pin<&mut Self>) {
            if let Some(ref format) = this.format {
                let render = |fmt: &mut fmt::Formatter<'_>| {
                    for unit in &format.0 {
                        unit.render(fmt, this.size, this.time)?;
                    }
                    Ok(())
                };

                if this.status_code >= 500 {
                    log::error!(
                        target: this.log_target.as_ref(),
                        "Access {}", FormatDisplay(&render)
                    );
                } else {
                    log::info!(
                        target: this.log_target.as_ref(),
                        "Access {}", FormatDisplay(&render)
                    );
                }
            }
        }
    }
}

impl<B: MessageBody> MessageBody for StreamLog<B> {
    type Error = B::Error;

    #[inline]
    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        let this = self.project();

        match ready!(this.body.poll_next(cx)) {
            Some(Ok(chunk)) => {
                *this.size += chunk.len();
                Poll::Ready(Some(Ok(chunk)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}

/// A formatting style for the `Logger` consisting of multiple concatenated `FormatText` items.
#[derive(Debug, Clone)]
struct Format(Vec<FormatText>);

impl Default for Format {
    /// Return the default formatting style for the `Logger`:
    fn default() -> Format {
        Format::new(r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#)
    }
}

impl Format {
    /// Create a `Format` from a format string.
    ///
    /// Returns `None` if the format string syntax is incorrect.
    pub fn new(s: &str) -> Format {
        log::trace!("Access log format: {}", s);
        let fmt = Regex::new(r"%(\{([A-Za-z0-9\-_]+)\}([aioe]|x[io])|[%atPrUsbTD]?)").unwrap();

        let mut idx = 0;
        let mut results = Vec::new();
        for cap in fmt.captures_iter(s) {
            let m = cap.get(0).unwrap();
            let pos = m.start();
            if idx != pos {
                results.push(FormatText::Str(s[idx..pos].to_owned()));
            }
            idx = m.end();

            if let Some(key) = cap.get(2) {
                results.push(match cap.get(3).unwrap().as_str() {
                    "a" => {
                        if key.as_str() == "r" {
                            FormatText::RealIpRemoteAddr
                        } else {
                            unreachable!("regex and code mismatch")
                        }
                    }
                    "i" => {
                        FormatText::RequestHeader(HeaderName::try_from(key.as_str()).unwrap())
                    }
                    "o" => {
                        FormatText::ResponseHeader(HeaderName::try_from(key.as_str()).unwrap())
                    }
                    "e" => FormatText::EnvironHeader(key.as_str().to_owned()),
                    "xi" => FormatText::CustomRequest(key.as_str().to_owned(), None),
                    "xo" => FormatText::CustomResponse(key.as_str().to_owned(), None),
                    _ => unreachable!(),
                })
            } else {
                let m = cap.get(1).unwrap();
                results.push(match m.as_str() {
                    "%" => FormatText::Percent,
                    "a" => FormatText::RemoteAddr,
                    "t" => FormatText::RequestTime,
                    "r" => FormatText::RequestLine,
                    "s" => FormatText::ResponseStatus,
                    "b" => FormatText::ResponseSize,
                    "U" => FormatText::UrlPath,
                    "T" => FormatText::Time,
                    "D" => FormatText::TimeMillis,
                    _ => FormatText::Str(m.as_str().to_owned()),
                });
            }
        }
        if idx != s.len() {
            results.push(FormatText::Str(s[idx..].to_owned()));
        }

        Format(results)
    }
}

/// A string of text to be logged.
///
/// This is either one of the data fields supported by the `Logger`, or a custom `String`.
#[non_exhaustive]
#[derive(Debug, Clone)]
enum FormatText {
    Str(String),
    Percent,
    RequestLine,
    RequestTime,
    ResponseStatus,
    ResponseSize,
    Time,
    TimeMillis,
    RemoteAddr,
    RealIpRemoteAddr,
    UrlPath,
    RequestHeader(HeaderName),
    ResponseHeader(HeaderName),
    EnvironHeader(String),
    CustomRequest(String, Option<CustomRequestFn>),
    CustomResponse(String, Option<CustomResponseFn>),
}

#[derive(Clone)]
struct CustomRequestFn {
    inner_fn: Rc<dyn Fn(&ServiceRequest) -> String>,
}

impl CustomRequestFn {
    fn call(&self, req: &ServiceRequest) -> String {
        (self.inner_fn)(req)
    }
}

impl fmt::Debug for CustomRequestFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("custom_request_fn")
    }
}

#[derive(Clone)]
struct CustomResponseFn {
    inner_fn: Rc<dyn Fn(&ServiceResponse) -> String>,
}

impl CustomResponseFn {
    fn call(&self, res: &ServiceResponse) -> String {
        (self.inner_fn)(res)
    }
}

impl fmt::Debug for CustomResponseFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("custom_response_fn")
    }
}

impl FormatText {
    fn render(
        &self,
        fmt: &mut fmt::Formatter<'_>,
        size: usize,
        entry_time: OffsetDateTime,
    ) -> Result<(), fmt::Error> {
        match self {
            FormatText::Str(ref string) => fmt.write_str(string),
            FormatText::Percent => "%".fmt(fmt),
            FormatText::ResponseSize => {
                size.human_count_bytes().fmt(fmt)
                // size.fmt(fmt)
            },
            FormatText::Time => {
                let rt = OffsetDateTime::now_utc() - entry_time;
                let rt = rt.as_seconds_f64();
                fmt.write_fmt(format_args!("{:.6}s", rt))
            }
            FormatText::TimeMillis => {
                let rt = OffsetDateTime::now_utc() - entry_time;
                // 1秒=1000毫秒(ms),          1毫秒=1／1000秒 
                // 1秒=1000000微秒(μs OR us), 1微秒=1／1000000秒
                // 1秒=1000000000纳秒(ns),    1纳秒=1／1000000000秒
                // 1秒=1000000000000皮秒,     1皮秒=1／1000000000000秒
                format_duration(Duration::from_nanos(rt.whole_nanoseconds() as u64)).fmt(fmt)
            }
            FormatText::EnvironHeader(ref name) => {
                if let Ok(val) = env::var(name) {
                    fmt.write_fmt(format_args!("{}", val))
                } else {
                    "-".fmt(fmt)
                }
            }
            _ => Ok(()),
        }
    }

    fn render_response(&mut self, res: &ServiceResponse) {
        match self {
            FormatText::ResponseStatus => {
                *self = FormatText::Str(format!("{}", res.status().as_u16()))
            }

            FormatText::ResponseHeader(ref name) => {
                let s = if let Some(val) = res.headers().get(name) {
                    if let Ok(s) = val.to_str() {
                        s
                    } else {
                        "-"
                    }
                } else {
                    "-"
                };
                *self = FormatText::Str(s.to_string())
            }

            FormatText::CustomResponse(_, res_fn) => {
                let text = match res_fn {
                    Some(res_fn) => FormatText::Str(res_fn.call(res)),
                    None => FormatText::Str("-".to_owned()),
                };

                *self = text;
            }

            _ => {}
        }
    }

    fn render_request(&mut self, now: OffsetDateTime, req: &ServiceRequest) {
        match self {
            FormatText::RequestLine => {
                *self = if req.query_string().is_empty() {
                    FormatText::Str(format!(
                        "{} {} {:?}",
                        req.method(),
                        req.path(),
                        req.version()
                    ))
                } else {
                    FormatText::Str(format!(
                        "{} {}?{} {:?}",
                        req.method(),
                        req.path(),
                        req.query_string(),
                        req.version()
                    ))
                };
            }
            FormatText::UrlPath => *self = FormatText::Str(req.path().to_string()),
            FormatText::RequestTime => *self = FormatText::Str(now.format(&Rfc3339).unwrap()),
            FormatText::RequestHeader(ref name) => {
                let s = if let Some(val) = req.headers().get(name) {
                    if let Ok(s) = val.to_str() {
                        s
                    } else {
                        "-"
                    }
                } else {
                    "-"
                };
                *self = FormatText::Str(s.to_string());
            }
            FormatText::RemoteAddr => {
                let s = if let Some(peer) = req.connection_info().peer_addr() {
                    FormatText::Str((*peer).to_string())
                } else {
                    FormatText::Str("-".to_string())
                };
                *self = s;
            }
            FormatText::RealIpRemoteAddr => {
                let s = if let Some(remote) = req.connection_info().realip_remote_addr() {
                    FormatText::Str(remote.to_string())
                } else {
                    FormatText::Str("-".to_string())
                };
                *self = s;
            }
            FormatText::CustomRequest(_, request_fn) => {
                let s = match request_fn {
                    Some(f) => FormatText::Str(f.call(req)),
                    None => FormatText::Str("-".to_owned()),
                };

                *self = s;
            }
            _ => {}
        }
    }
}

/// Converter to get a String from something that writes to a Formatter.
pub(crate) struct FormatDisplay<'a>(
    &'a dyn Fn(&mut fmt::Formatter<'_>) -> Result<(), fmt::Error>,
);

impl<'a> fmt::Display for FormatDisplay<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        (self.0)(fmt)
    }
}

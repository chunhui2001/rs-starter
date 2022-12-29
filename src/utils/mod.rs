pub mod file;
pub mod parse;

// println!("{}", type_of(&1));
// println!("{}", type_of(&1.434));
// println!("{}", type_of(&""));
// println!("{}", type_of(&s1));
// println!("{}", type_of(&{ || "Hi!" }));
// println!("{}", type_of(&type_of::<i32>));
pub fn type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

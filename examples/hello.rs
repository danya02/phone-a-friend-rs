extern crate phone_a_friend;
use phone_a_friend::*;
use rand::Rng;

phone_a_friend_telegram! {
[token="xxxxx:yyyyyyyyyy", chat_id=1234512345]
    #[derive(Debug)]
    struct Point {
        x: PhoneAFriend("What should the type of `x` be in my `Point` struct? It needs to be something that the `rand` crate can generate for me."),
        y: PhoneAFriend("What about the type of `y`?"),
    }
}

impl Point {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        Point {
            x: rng.gen(),
            y: rng.gen(),
        }
    }
}

/// Get the name of a type using std::any::type_name,
/// in a way that's like std::any::type_name_of_val.
fn type_name<T>(_value: T) -> String {
    std::any::type_name::<T>().to_string()
}

fn main() {
    println!("My friend helped me to write my struct: ");
    let new_point = Point::new();
    println!("struct Point {{");
    println!("  x: {},", type_name(new_point.x));
    println!("  y: {},", type_name(new_point.y));
    println!("}}");
    println!();
    println!("Here's an example of this struct: {new_point:?}");

}
extern crate phone_a_friend;
use phone_a_friend::*;

#[phone_a_friend(a=b, c=d)]
struct Foo {
    a: i32,
    b: PhoneAFriend<"What is it called?">,
}

fn main() {
}
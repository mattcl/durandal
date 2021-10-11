pub mod cli;
pub mod routine;

pub async fn say(msg: &str) {
    println!("{}", msg);
}

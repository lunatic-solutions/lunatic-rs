use lunatic::net;

fn main() {
    let wikipedia = net::resolve("wikipedia.org:80").unwrap();
    for addr in wikipedia {
        println!("{:?}", addr);
    }
}

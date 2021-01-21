use lunatic::Process;

fn main() {
    Process::spawn_with((), |_: ()| {
        println!("Hello from inside the new process!");
    })
    .join()
    .unwrap();
}

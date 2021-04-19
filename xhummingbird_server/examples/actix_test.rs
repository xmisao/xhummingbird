use actix::prelude::*;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use std::time::Duration;
use std::thread;
use tokio::time::sleep;

fn main() {
    let sys = actix::System::new("app");

    HttpServer::new(move ||
                    App::new()
                    .service(root)
                   ).bind("0.0.0.0:8080").unwrap().disable_signals().run();

    let mut arbiter0 = Arbiter::new();
    let shutdown_actor = ShutdownActor::new();
    let shutdown_actor_address = ShutdownActor::start_in_arbiter(&arbiter0, |_| shutdown_actor);

    let mut arbiter1 = Arbiter::new();
    let sleep_actor = SleepActor{shutdown_actor_address: shutdown_actor_address.clone()};
    let sleep_actor_address = SleepActor::start_in_arbiter(&arbiter1, |_| sleep_actor);
    let sleep_actor_address1 = sleep_actor_address.clone();

    let mut arbiter2 = Arbiter::new();
    let sleep_actor2 = SleepActor2{shutdown_actor_address: shutdown_actor_address.clone()};
    let sleep_actor2_address = SleepActor2::start_in_arbiter(&arbiter2, |_| sleep_actor2);
    let sleep_actor2_address1 = sleep_actor2_address.clone();

    actix::spawn(async move {
        for n in 0..10 {
            sleep_actor_address1.try_send(SleepInstruction{n});
            println!("Sent instruction {}", n);
        }

        for n in 0..20 {
            sleep_actor2_address1.try_send(SleepInstruction{n});
            println!("Sent instruction {}", n);
        }
    });

    ctrlc::set_handler(move || {
        let sleep_actor_address2 = sleep_actor_address.clone();
        let sleep_actor2_address2 = sleep_actor2_address.clone();

        println!("ctrlc handler started");

        println!("Send stop to actor: {:?}", sleep_actor_address2.try_send(Stop{}).is_ok());
        println!("Send stop to actor2: {:?}", sleep_actor2_address2.try_send(Stop{}).is_ok());

        println!("ctrlc handler finished");
    }).unwrap();

    // System::current().stop();

    let _ = sys.run();
    println!("sys.run() finished.");

    arbiter2.join();
    println!("arbiter2.join() finished.");
    arbiter1.join();
    println!("arbiter1.join() finished.");
    arbiter0.join();
    println!("arbiter0.join() finished.");

    println!("exit");
}

#[get("/")]
async fn root() -> impl Responder {
    "Hello"
}

struct SleepActor{
    pub shutdown_actor_address: Addr<ShutdownActor>
}

impl Actor for SleepActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(100);
    }

    fn stopped(&mut self, ctx: &mut Self::Context){
        println!("SA1: stopped");
        let result = self.shutdown_actor_address.try_send(Stopped{actor_id: 1});
        println!("{:?}", result);
    }
}

impl Handler<SleepInstruction> for SleepActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: SleepInstruction, _ctx: &mut Context<Self>) -> Self::Result {
        println!("SA1: sleeping #{}", msg.n);
        thread::sleep(Duration::from_millis(1000));
        println!("SA1: sleeped");
        Ok(())
    }
}

impl Handler<Stop> for SleepActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: Stop, ctx: &mut Context<Self>) -> Self::Result {
        println!("SA1: Stop handler started.");

        Context::stop(ctx);

        println!("SA1: Stop handler finished.");

        Ok(())
    }
}

struct SleepActor2{
    pub shutdown_actor_address: Addr<ShutdownActor>
}

impl Actor for SleepActor2{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(100);
    }

    fn stopped(&mut self, ctx: &mut Self::Context){
        println!("SA2: stopped");
        let result = self.shutdown_actor_address.try_send(Stopped{actor_id: 2});
        println!("{:?}", result);
    }
}

impl Handler<SleepInstruction> for SleepActor2 {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: SleepInstruction, _ctx: &mut Context<Self>) -> Self::Result {
        println!("SA2: sleeping #{}", msg.n);
        thread::sleep(Duration::from_millis(1000));
        println!("SA2: sleeped");
        Ok(())
    }
}

impl Handler<Stop> for SleepActor2 {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: Stop, ctx: &mut Context<Self>) -> Self::Result {
        println!("SA2: Stop handler started.");

        Context::stop(ctx);

        println!("SA2: Stop handler finished.");

        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct SleepInstruction{
    pub n:u32
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct Stop{
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct Stopped{
    pub actor_id:u32
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct Shutdown{
}

struct ShutdownActor{
    stopped_actor_ids: Vec<u32>
}

impl ShutdownActor{
    pub fn new() -> ShutdownActor{
        ShutdownActor{stopped_actor_ids: vec!()}
    }

    pub fn join(&mut self, actor_id: u32) -> bool{
        self.stopped_actor_ids.push(actor_id);
        self.stopped_actor_ids.contains(&1) && self.stopped_actor_ids.contains(&2)
    }
}

impl Actor for ShutdownActor{
    type Context = Context<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context){
        println!("Shutdown Actor: stopped");
        System::current().stop();
    }
}

impl Handler<Stopped> for ShutdownActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: Stopped, ctx: &mut Context<Self>) -> Self::Result {
        println!("Shutdown handler started.");
        if self.join(msg.actor_id) {
            println!("Joined!");
            Context::stop(ctx);

            //std::process::exit(0); // Added
        } else {
            println!("Yet joined.");
        }
        println!("Shutdown handler finished.");

        Ok(())
    }
}

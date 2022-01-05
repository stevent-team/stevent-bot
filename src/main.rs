extern crate dotenv;

use std::env;
use std::time::Duration;
use dotenv::dotenv;
use tokio_cron_scheduler::{JobScheduler, Job};

use serenity::{
    utils,
    async_trait,
    model::{channel::Message, gateway::Ready, guild::PartialGuild},
    prelude::*,
};

// Schedules
const LIGHT_THEME_SCHEDULE: &str = "0 0 07 * * * *";
const DARK_THEME_SCHEDULE: &str  = "0 0 17 * * * *";

// Icons
const LIGHT_ICON: &str = "./Stevent_Logo_Light.png";
const DARK_ICON: &str = "./Stevent_Logo_Dark.png";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("\"{}\" is connected!", ready.user.name);

        // Schedule icon updates
        schedule_icon_updates(ctx, ready).await;
    }
}

async fn schedule_icon_updates(ctx: Context, ready: Ready) {
    // Create a scheduler
    let mut sched = JobScheduler::new();    

    // Create a light-theme job
    let light_ctx = ctx.clone();
    let light_ready = ready.clone();
    let light_job = Job::new_async(LIGHT_THEME_SCHEDULE, move |_uuid, _l| {
        let light_ctx = light_ctx.clone();
        let light_ready = light_ready.clone();
        Box::pin(async move {
            println!("Applying light theme!");
            update_guild_icons(&light_ctx, &light_ready, LIGHT_ICON).await;
        })
    }).unwrap();

    // Create a dark theme job
    let dark_ctx = ctx.clone();
    let dark_ready = ready.clone();
    let dark_job = Job::new_async(DARK_THEME_SCHEDULE, move |_uuid, _l| {
        let dark_ctx = dark_ctx.clone();
        let dark_ready = dark_ready.clone();
        Box::pin(async move {
            println!("Applying dark theme!");
            update_guild_icons(&dark_ctx, &dark_ready, DARK_ICON).await;
        })
    }).unwrap();

    // Schedule the jobs
    sched.add(dark_job).unwrap();
    sched.add(light_job).unwrap();

    loop {
        sched.tick().unwrap();
        std::thread::sleep(Duration::from_millis(500));
    }
}

async fn update_guild_icons(ctx: &Context, ready: &Ready, icon_path: &str) {
    // Update the icon of each guild the bot is in
    if let Ok(guilds) = ready.user.guilds(&ctx.http).await {
        for guild_info in guilds {
            if let Ok(mut guild) = ctx.http.get_guild(guild_info.id.0).await {
                update_guild_icon(&ctx, &mut guild, icon_path).await;
            }
        }
    }
}

async fn update_guild_icon(ctx: &Context, guild: &mut PartialGuild, icon_path: &str) {
    // Read icon
    let base64_icon = utils::read_image(icon_path)
        .expect("Failed to read specified guild icon.");

    // Update icon
    if let Err(why) = guild.edit(&ctx.http, |g| g.icon(Some(&base64_icon))).await {
        println!("Couldn't edit guild: {}", why);
    };
} 

#[tokio::main]
async fn main() {
    // Load env from .env
    dotenv().ok();

    // Get the token from the env
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a client
    let mut client =
        Client::builder(&token).event_handler(Handler).await.expect("Failed to create client");

    // Start a shard
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

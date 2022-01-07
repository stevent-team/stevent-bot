extern crate dotenv;

use std::env;
use dotenv::dotenv;
use std::net::{TcpListener, TcpStream};
use std::io::{Read};
use serenity::{
    utils,
    async_trait,
    model::{channel::Message, gateway::Ready, guild::PartialGuild},
    prelude::*,
};

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
        start_server(&ctx, &ready).await;
    }
}

async fn start_server(ctx: &Context, ready: &Ready) {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on 3333");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection from {}", stream.peer_addr().unwrap());
                handle_client(stream, &ctx, &ready).await;
            }
            Err(err) => {
                println!("Connection failed: {}", err)
            }
        }
    }

    // Close the socket
    drop(listener);
}

async fn handle_client(mut stream: TcpStream, ctx: &Context, ready: &Ready) {
    let mut data = [0 as u8; 50];
    if let Ok(size) = stream.read(&mut data) {
        match &data[0..size] {
            b"light" => { update_guild_icons(&ctx, &ready, LIGHT_ICON).await; }
            b"dark" => { update_guild_icons(&ctx, &ready, DARK_ICON).await; }
            _ => { println!("Unknown option") }
        };
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

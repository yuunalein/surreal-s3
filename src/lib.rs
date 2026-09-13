#![feature(custom_inner_attributes)]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(int_roundings)]

use surrealism::surrealism;

mod aws_sdk;
mod config;
mod dns;
mod http;
mod result;
mod util;

mod multipart;
mod sign;

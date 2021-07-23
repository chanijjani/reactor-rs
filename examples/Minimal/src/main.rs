//-- Generated by LFC @ 2021/07/23 16:06:15 --//
#![allow(unused_imports)]
#![allow(non_snake_case)]

#[macro_use]
extern crate reactor_rt;

mod reactors;

use ::reactor_rt::*;
use self::reactors::MinimalAssembler as _MainAssembler;
use self::reactors::MinimalParams as _MainParams;

fn main() {
    let options = SchedulerOptions {
        timeout: None,
        keep_alive: false,
    };
    let main_args = _MainParams { /* main params are de facto forbidden */ };

    SyncScheduler::run_main::<_MainAssembler>(options, main_args);
}

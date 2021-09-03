//-- Generated by LFC @ 2021/07/26 14:57:03 --//
#![allow(unused)]

use ::reactor_rt::{LogicalInstant, PhysicalInstant, Duration};
use ::reactor_rt::Offset::{After, Asap};
use std::sync::{Arc, Mutex};



// todo link to source
pub struct Minimal {

}

#[warn(unused)]
impl Minimal {

    // --- reaction(startup) {= ... =}
    fn react_0(&mut self,
               #[allow(unused)] ctx: &mut ::reactor_rt::ReactionCtx,
               #[allow(unused)] params: &MinimalParams,
    ) {
        println!("Hello World.");
    }

}

/// Parameters for the construction of a [Minimal]
#[derive(Clone)]
pub struct MinimalParams {

}


//------------------------//


pub struct MinimalDispatcher {
    _id: ::reactor_rt::ReactorId,
    _impl: Minimal,
    _params: MinimalParams,
    _startup_reactions: ::reactor_rt::ReactionSet,
    _shutdown_reactions: ::reactor_rt::ReactionSet,

}

impl MinimalDispatcher {
    #[inline]
    fn user_assemble(_id: ::reactor_rt::ReactorId, args: MinimalParams) -> Self {
        let MinimalParams {  } = args.clone();
        Self {
            _id,
            _params: args,
            _startup_reactions: Default::default(),
            _shutdown_reactions: Default::default(),
            _impl: Minimal {

            },

        }
    }
}

use ::reactor_rt::*; // after this point there's no user-written code

impl ::reactor_rt::ReactorInitializer for MinimalDispatcher {
    type Wrapped = Minimal;
    type Params = MinimalParams;
    const MAX_REACTION_ID: LocalReactionId = LocalReactionId::new_const(1);

    fn assemble(args: Self::Params, assembler: &mut AssemblyCtx) -> Self {
        // children reactors   


        // assemble self
        let mut _self = Self::user_assemble(assembler.get_next_id(), args);

        let react_0 = GlobalReactionId::new(_self.id(), 0.into());

        {
            _self._startup_reactions = vec![react_0,];
            _self._shutdown_reactions = vec![];


        }
        {
            // Declare connections
        }


       _self
    }
}


impl ::reactor_rt::ReactorBehavior for MinimalDispatcher {

    #[inline]
    fn id(&self) -> ReactorId {
        self._id
    }

    fn react_erased(&mut self, ctx: &mut ::reactor_rt::ReactionCtx, rid: LocalReactionId) {
        match rid.index() {
            0 => self._impl.react_0(ctx, &self._params),

            _ => panic!("Invalid reaction ID: {} should be < {}", rid, Self::MAX_REACTION_ID)
        }
    }

    fn cleanup_tag(&mut self, ctx: &::reactor_rt::CleanupCtx) {

    }
    
    fn enqueue_startup(&self, ctx: &mut StartupCtx) {
        ctx.enqueue(&self._startup_reactions);

    }

    fn enqueue_shutdown(&self, ctx: &mut StartupCtx) {
        ctx.enqueue(&self._shutdown_reactions);
    }

}

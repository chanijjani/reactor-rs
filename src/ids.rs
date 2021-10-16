/*
 * Copyright (c) 2021, TU Dresden.
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL
 * THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
 * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF
 * THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */


use core::any::type_name;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter, Result, Write};
use std::hash::{Hash, Hasher};
use std::ops::Range;

use index_vec::IndexVec;

use crate::ReactorInitializer;

// private implementation types
type ReactionIdImpl = u16;
type ReactorIdImpl = u16;
pub(in crate) type GlobalIdImpl = u32;

define_index_type! {
    /// Type of a local reaction ID
    pub struct LocalReactionId = ReactionIdImpl;
    DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
    DISPLAY_FORMAT = "{}";
}

impl LocalReactionId {
    pub const ZERO: LocalReactionId = LocalReactionId::new_const(0);

    // a const fn to be able to use this in const context
    pub const fn new_const(u: ReactionIdImpl) -> Self {
        Self { _raw: u }
    }
}


define_index_type! {
    /// The unique identifier of a reactor instance during
    /// execution.
    pub struct ReactorId = ReactorIdImpl;
    DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
    DISPLAY_FORMAT = "{}";
    DEFAULT = Self::new(0);
}

impl ReactorId {
    // a const fn to be able to use this in const context
    pub const fn new_const(u: ReactorIdImpl) -> Self {
        Self { _raw: u }
    }
}

macro_rules! global_id_newtype {
    {$(#[$m:meta])* $id:ident} => {
        $(#[$m])*
        #[derive(Eq, Ord, PartialOrd, PartialEq, Hash, Copy, Clone)]
        pub struct $id(pub(in crate) GlobalId);

        impl $id {
            pub fn new(container: $crate::ReactorId, local: $crate::LocalReactionId) -> Self {
                Self($crate::GlobalId::new(container, local))
            }
        }

        impl Debug for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

global_id_newtype! {
    /// Global identifier for a reaction.
    GlobalReactionId
}

#[derive(Debug, Eq, PartialEq)]
pub struct TriggerId(pub(crate) TriggerIdImpl);

impl TriggerId {
    pub const STARTUP: TriggerId = TriggerId(TriggerIdImpl::Startup);
    pub const SHUTDOWN: TriggerId = TriggerId(TriggerIdImpl::Shutdown);

    pub(crate) fn new(id: GlobalId) -> Self {
        TriggerId(TriggerIdImpl::Component(id))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TriggerIdImpl {
    Startup,
    Shutdown,
    Component(GlobalId),
}


impl Hash for TriggerId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // this hash function is very hot (because of get_reactions_trigerred_by)
        // so we give it an implementation that's basically free.
        let h: u32 = match self.0 {
            // we don't care about collisions, esp bc they occur once per app
            TriggerIdImpl::Startup | TriggerIdImpl::Shutdown => u32::MAX,
            TriggerIdImpl::Component(GlobalId { _raw }) => _raw
        };
        state.write_u32(h)
    }
}
//
// global_id_newtype! {
//     /// Global identifier for a trigger (port, action, timer)
//     ComponentId
// }


/// Identifies a component of a reactor using the ID of its container
/// and a local component ID.
#[derive(Eq, Ord, PartialOrd, PartialEq, Copy, Clone)]
pub(crate) struct GlobalId {
    _raw: GlobalIdImpl,
}


impl GlobalId {
    pub fn new(container: ReactorId, local: LocalReactionId) -> Self {
        let _raw: GlobalIdImpl = (container._raw as GlobalIdImpl) << ReactionIdImpl::BITS | (local._raw as GlobalIdImpl);
        Self { _raw }
    }

    // fixme replace panic by AssemblyError

    pub(crate) fn next_id(&self) -> GlobalId {
        assert_ne!(self.local(), 0xffff, "Overflow while allocating next id");
        Self { _raw: self._raw + 1 }
    }

    pub(crate) fn id_range(&self, len: usize) -> Range<GlobalId> {
        match ReactionIdImpl::try_from(self.local().index() + len) {
            Ok(_) =>
                Range { start: *self, end: Self { _raw: self._raw + (len as GlobalIdImpl) } }
            ,
            Err(..) => panic!("Overflow while creating ID range"),
        }
    }

    #[cfg(test)]
    pub const fn first_id() -> GlobalId {
        GlobalId { _raw: 0 }
    }

    pub(in crate) const fn container(&self) -> ReactorId {
        ReactorId::new_const((self._raw >> 16) as u16)
    }

    pub(in crate) const fn local(&self) -> LocalReactionId {
        LocalReactionId::new_const((self._raw & 0xffff) as u16)
    }
}

// todo commit and remove
#[cfg(nightly)]
impl std::iter::Step for GlobalId {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        (end._raw as usize).checked_sub(start._raw as usize)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        GlobalIdImpl::try_from(count).ok()
            .and_then(|c| start._raw.checked_add(c))
            .map(|_raw| Self { _raw })
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        GlobalIdImpl::try_from(count).ok()
            .and_then(|c| start._raw.checked_sub(c))
            .map(|_raw| Self { _raw })
    }
}

impl Hash for GlobalId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self._raw)
    }
}

impl Debug for GlobalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for GlobalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}/{}", self.container(), self.local())
    }
}


pub(in crate) trait GloballyIdentified {
    fn get_id(&self) -> GlobalId;
}

pub(crate) type PortId = GlobalId;

#[derive(Clone)]
pub(in crate) struct ReactorDebugInfo {
    /// Type name
    #[allow(unused)]
    pub type_name: &'static str,
    /// Simple name of the instantiation (last segment of the path)
    #[allow(unused)]
    pub inst_name: &'static str,
    /// Path to this instantiation (eg "/parent/child")
    inst_path: String,
}

impl ReactorDebugInfo {
    pub(in crate) fn root<R>() -> Self {
        Self {
            type_name: type_name::<R>(),
            inst_name: "/",
            inst_path: "/".into(),
        }
    }

    pub(in crate) fn derive<R: ReactorInitializer>(&self, inst_name: &'static str) -> Self {
        Self {
            type_name: type_name::<R::Wrapped>(),
            inst_name,
            inst_path: format!("{}{}/", self.inst_path, inst_name),
        }
    }
}

impl Display for ReactorDebugInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.inst_path)
    }
}


/// Stores a mapping from global Id to debug label
#[derive(Default)]
pub(in crate) struct IdRegistry {
    debug_ids: HashMap<GlobalId, Cow<'static, str>>,
    reactor_infos: IndexVec<ReactorId, ReactorDebugInfo>,
}

impl IdRegistry {
    pub fn get_debug_label(&self, id: GlobalId) -> Option<&str> {
        self.debug_ids.get(&id).map(Cow::as_ref)
    }

    pub fn get_debug_info(&self, id: ReactorId) -> &ReactorDebugInfo {
        &self.reactor_infos[id]
    }

    fn fmt_component_path(&self, id: GlobalId) -> String {
        format!("{}{}", self.get_debug_info(id.container()), id.local())
    }

    #[cfg(feature = "graph-dump")]
    pub(crate) fn fmt_component(&self, id: GlobalId) -> String {
        if let Some(label) = self.get_debug_label(id) {
            format!("{}{}", self.get_debug_info(id.container()), label)
        } else {
            self.fmt_component_path(id)
        }
    }

    #[inline]
    pub fn fmt_reaction(&self, id: GlobalReactionId) -> String {
        let mut str = self.fmt_component_path(id.0);
        // reactions may have labels too
        if let Some(label) = self.get_debug_label(id.0) {
            write!(str, "@{}", label).unwrap();
        }
        str
    }

    pub(in super) fn record(&mut self, id: GlobalId, name: Cow<'static, str>) {
        let existing = self.debug_ids.insert(id, name);
        debug_assert!(existing.is_none())
    }

    pub(in super) fn record_reactor(&mut self, id: ReactorId, debug: &ReactorDebugInfo) {
        let ix = self.reactor_infos.push(debug.clone());
        debug_assert_eq!(ix, id);
    }
}

use std::{collections::HashMap, sync::{Arc, Mutex}, time::{UNIX_EPOCH, SystemTime, Duration}};
use crate::{data::{ClassPageData, ClassEntry, ClassItemEntryContents}, ClassID, ClassItemID};

pub type AMutComponent<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone)]
pub struct TimedComponent<T> {
    pub prev_update: Arc<Mutex<SystemTime>>,
    pub data: T,
}

const TIMEOUT: Duration = Duration::from_secs(60 * 10);

impl<T> Default for TimedComponent<T>
where T: Default {
    fn default() -> TimedComponent<T> {
        TimedComponent {
            prev_update: Arc::new(Mutex::new(UNIX_EPOCH)),
            data: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BackendCache {
    pub class_listing: TimedComponent<AMutComponent<Option<Vec<ClassEntry>>>>,
    pub class_data: AMutComponent<HashMap<ClassID, TimedComponent<ClassPageData>>>,
    pub assignment_data: TimedComponent<Option<HashMap<ClassItemID, ClassItemEntryContents>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheDataState {
    Ok,
    Stale,
    None,
}

impl BackendCache {
    pub const fn get_timeout() -> Duration {
        TIMEOUT
    }

    pub fn get_class_listing_state(&self) -> CacheDataState {
        if self
            .class_listing
            .data
            .lock()
            .unwrap()
            .is_some() {
            if SystemTime::now()
                .duration_since(
                    self
                        .class_listing
                        .prev_update
                        .lock()
                        .ok()
                        .map(|guard| *guard)
                        .unwrap_or(UNIX_EPOCH)
                )
                .unwrap_or(BackendCache::get_timeout()) < BackendCache::get_timeout() {
                CacheDataState::Ok
            } else {
                CacheDataState::Stale
            }
        } else {
            CacheDataState::None
        }
    }

    pub fn get_class_data_state(&self, id: ClassID) -> CacheDataState {
        if let Some(value) = self
            .class_data
            .lock()
            .unwrap()
            .get(&id) {
            if SystemTime::now()
                .duration_since(
                    value
                        .prev_update
                        .lock()
                        .ok()
                        .map(|guard| *guard)
                        .unwrap_or(UNIX_EPOCH)
                )
                .unwrap_or(BackendCache::get_timeout()) < BackendCache::get_timeout() {
                CacheDataState::Ok
            } else {
                CacheDataState::Stale
            }
        } else {
            CacheDataState::None
        }
    }

    // fn get_class_listing_state(&self) -> CacheDataState {
    //     if self
    //         .class_listing
    //         .data
    //         .lock()
    //         .unwrap()
    //         .is_some() {
    //         if SystemTime::now()
    //             .duration_since(self.class_listing.prev_update)
    //             .unwrap_or(BackendCache::get_timeout()) < BackendCache::get_timeout() {
    //             CacheDataState::Ok
    //         } else {
    //             CacheDataState::Stale
    //         }
    //     } else {
    //         CacheDataState::None
    //     }
    // }
}
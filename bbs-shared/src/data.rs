use std::{collections::HashMap, rc::Rc, cell::RefCell, ops::{DerefMut, Deref}, num::ParseIntError};
use std::cmp::Ordering::*;

use serde::{Serialize, Deserialize};
use yew::Properties;

use crate::{ClassID, ClassItemID, DueDate, add_base64};

pub type OptMutComponent<T> = Rc<RefCell<Option<T>>>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Keyed<T>(T, usize);

impl<T> Keyed<T> {
    pub fn new_inc(&self, new_val: T) -> Self {
        Keyed(new_val, self.1 + 1)
    }

    pub fn value(self) -> T {
        self.0
    }

    pub fn key(&self) -> usize {
        self.1
    }
}

impl<T> Keyed<T>
where T: Clone {
    pub fn new_inc_clone(&self) -> Self {
        Keyed(self.0.clone(), self.1 + 1)
    }
}

impl<T> Deref for Keyed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FrontendData {
    pub classes: Keyed<OptMutComponent<Vec<ClassEntry>>>,
    pub curr_class_data: Keyed<OptMutComponent<ClassPageData>>,
}

impl FrontendData {
    pub fn empty() -> Self {
        Self {
            classes: Keyed(Rc::new(RefCell::new(None)), 0),
            curr_class_data: Keyed(Rc::new(RefCell::new(None)), 0),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Properties)]
pub struct ClassEntry {
    pub name: String,
    pub section: SectionData,
    pub picture: Vec<u8>,
    pub id: ClassID,
}

impl PartialOrd for ClassEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match (&self.section.guts, &other.section.guts) {
            (SectionDataGuts::Good {
                period: self_period,
                days: self_days,
            }, SectionDataGuts::Good {
                period: other_period,
                days: other_days,
            }) => match self_period.cmp(other_period) {
                Equal => self_days
                    .iter()
                    .zip(other_days)
                    .filter_map(|(s, o)| match o.cmp(s) {
                        Equal => None,
                        order => Some(order)
                    })
                    .next()
                    .unwrap_or(Equal),
                order => order,
            },
            (SectionDataGuts::Good {..}, &SectionDataGuts::Bad(_)) => Less,
            (SectionDataGuts::Bad(_), &SectionDataGuts::Good {..}) => Greater,
            _ => Equal
        })
    }
}

impl Ord for ClassEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

add_base64! { ClassEntry }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Properties)]
pub struct SectionData {
    pub guts: SectionDataGuts,
}

impl From<&str> for SectionData {
    fn from(s: &str) -> Self {
        
        match SectionDataGuts::try_good_from_str(s) {
            Ok(guts) => Self { guts },
            Err(e) => {
                eprintln!("{}", e);
                Self { guts: SectionDataGuts::Bad(s.to_owned()) }
            },
        }
    }
}

impl Deref for SectionData {
    type Target = SectionDataGuts;

    fn deref(&self) -> &Self::Target {
        &self.guts
    }
}
impl DerefMut for SectionData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SectionDataGuts {
    Bad(String),
    Good {
        days: [bool; 7],
        period: u64,
    }
}

impl SectionDataGuts {
    fn try_good_from_str(s: &str) -> Result<Self, String> {
        let paren_location = s.find('(').ok_or("Failed to find opening parenthesis!")?;
        
        let period_slice = &s[0..paren_location];

        let period = period_slice.parse().map_err(|err: ParseIntError| err.to_string())?;
        
        let days_slice = &s[paren_location+1..s.len()-1];

        let days = (| | {
            const DICT: &str = "ABCDE";

            let mut days = [false; 7];
            
            days_slice
                .split(',')
                .filter_map(|day_days| {
                    day_days
                        .split_once('-')
                        .map_or(
                            Some((|n| n..n+1)(DICT.find(day_days.chars().nth(0)?)?)),
                            |(start, end)| Some(DICT.find(start.chars().nth(0)?)?..DICT.find(end.chars().nth(0)?)? + 1),
                        )
                })
                .flatten()
                .for_each(|day_num| days[day_num + 1] = true);

            Some(days)
        })().unwrap_or([false; 7]);

        Ok(Self::Good {
            period,
            days,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Properties)]
pub struct ClassPageData {
    id: ClassID,
    entry_id_map: HashMap<ClassItemID, ClassItemEntryData>,
    hierarchy: Vec<ClassItemEntryData>,
}

add_base64! { ClassPageData }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassItemEntryData {
    name: String,
    contents: ClassItemEntryContents,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClassItemEntryContents {
    Assignment {
        preview_text: String,
        due: DueDate,
    },
    Discussion {
        preview_text: String,
    },
    TestQuiz {
        due: DueDate,
    },
    File {
        file_type: (),
    },

    Folder {
        contained: Option<Vec<ClassItemID>>
    },

    Other {},
}

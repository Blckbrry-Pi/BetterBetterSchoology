use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveClasses {
    pub response_code: u64,
    pub body: ActiveClassesBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveClassesBody {
    pub courses: ActiveClassesCourses,
    pub permissions: ActiveClassesPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveClassesCourses {
    pub courses: Vec<ActiveClassesCourse>,
    pub sections: Vec<ActiveClassesSection>
}

impl ActiveClassesCourses {
    pub fn to_by_id(&self) -> ClassesById {
        let mut simple_courses: HashMap<u64, ActiveClassesCourse> = self
            .courses
            .iter()
            .map(|course| (course.nid, course.clone()))
            .collect();

        let full_courses = self
            .sections
            .iter()
            .filter_map(|section| Some((section.course_nid, (simple_courses.remove(&section.course_nid)?, section.clone()))))
            .collect();

        ClassesById {
            data: full_courses
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveClassesCourse {
    pub nid: u64,
    pub course_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveClassesSection {
    pub nid: u64,

    pub section_title: String,
    pub course_nid: u64,
    pub logo_img_src: ActiveClassesThumbnails,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveClassesThumbnails {
    pub card_thumbnail: Option<String>,
    pub card_thumbnail_2x: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveClassesPermissions {
    pub is_verified: bool,
    pub can_browse_courses: bool,
    pub can_join_courses: bool,
    pub can_create_courses: bool,
    pub school_has_grading_periods: bool,
}

pub struct ClassesById {
    pub data: HashMap<u64, (ActiveClassesCourse, ActiveClassesSection)>,
}
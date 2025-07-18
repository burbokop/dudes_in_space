use crate::CORE_PACKAGE_ID;
use crate::modules::{Assembler, PersonnelArea};
use dudes_in_space_api::Vessel;
use dudes_in_space_api::modules::Module;

pub(crate) trait CoreModule: Module {
    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()>;
    fn accept_visitor_mut(&mut self, v: &dyn ModuleVisitorMut<Result = ()>) -> Option<()>;
}

pub(crate) trait ModuleVisitor {
    type Result;

    fn visit_personnel_area(&self, _: &PersonnelArea) -> Option<Self::Result> {
        None
    }

    fn visit_assembler(&self, _: &Assembler) -> Option<Self::Result> {
        None
    }
}

pub trait ModuleVisitorMut {
    type Result;

    fn visit_personnel_area(&self, _: &mut PersonnelArea) -> Option<Self::Result> {
        None
    }

    fn visit_assembler(&self, _: &mut Assembler) -> Option<Self::Result> {
        None
    }
}

pub trait VisitModules {
    fn visit_modules(&self, visitor: &dyn ModuleVisitor<Result = ()>) -> Option<()>;
    fn visit_modules_mut(&mut self, visitor: &dyn ModuleVisitorMut<Result = ()>) -> Option<()>;
}

// impl VisitModules for Vessel {
//     fn visit_modules(&self, visitor: &dyn ModuleVisitor<Result = ()>) -> Option<()> {
//         for module in self.modules() {
//             unsafe {
//                 if module.package_id() == CORE_PACKAGE_ID {
//                     let ptr = module.deref().deref() as *const dyn Module;
//
//
//
//                     let casted_ptr: *const dyn CoreModule = std::mem::transmute(ptr);
//                     if let Some(r) = (&*casted_ptr).accept_visitor(visitor) {
//                         return Some(r);
//                     }
//                 }
//             }
//         }
//         None
//     }
//
//     fn visit_modules_mut(&mut self, visitor: &dyn ModuleVisitorMut<Result = ()>) -> Option<()> {
//         for mut module in self.modules_mut() {
//             unsafe {
//                 if module.package_id() == CORE_PACKAGE_ID {
//                     let rf = module.deref_mut().deref_mut();
//                     let ptr = rf as *mut dyn Module;
//                     let casted_ptr: *mut dyn CoreModule = std::mem::transmute(ptr);
//
//                     if let Some(r) = (&mut *casted_ptr).accept_visitor_mut(visitor) {
//                         return Some(r);
//                     }
//                 }
//             }
//         }
//         None
//     }
// }

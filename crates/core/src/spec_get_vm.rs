use crate::{FromOdraValue, Vm};

pub struct VmToken;
pub struct OtherToken;

pub trait GetVmForVm {
    fn get_access_token(&self) -> VmToken {
        VmToken
    }
}

impl GetVmForVm for std::marker::PhantomData<&mut Vm> {}

pub trait GetVmForOther {
    fn get_access_token(&self) -> OtherToken {
        OtherToken
    }
}

impl<T: FromOdraValue> GetVmForOther for &std::marker::PhantomData<T> {}

impl VmToken {
    pub fn get<'a>(self, vm: &'a mut Vm) -> &'a mut Vm {
        vm
    }
}

impl OtherToken {
    pub fn get<T: FromOdraValue>(self, vm: &mut Vm) -> T {
        vm.pop_from_stack()
    }
}

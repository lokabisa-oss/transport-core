use crate::auth::AuthState;

#[repr(C)]
pub struct transport_core_client {
    auth_state: AuthState,
}

#[no_mangle]
pub extern "C" fn tc_client_new() -> *mut transport_core_client {
    Box::into_raw(Box::new(transport_core_client {
        auth_state: AuthState::new(),
    }))
}

#[no_mangle]
pub extern "C" fn tc_client_free(ptr: *mut transport_core_client) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

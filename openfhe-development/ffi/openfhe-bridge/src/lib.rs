pub fn create_threshold_context() -> *mut c_void {
    unsafe { crate::openfhe_bridge::create_threshold_context() }
}

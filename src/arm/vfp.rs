use core::intrinsics;

// Thumb1 code can't encode hardware float operations, so some targets
// need functions that wrap the appropriate ARM instructions.
// `thumbv7em` has vfp but only for 32-bit floats.

// Mathematics functions

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __adddf3vfp() {
    #[cfg(armhf)]
    asm!("vadd.f64 d0, d0, d1");
    #[cfg(not(armhf))]
    asm!("vmov d6, r0, r1
          vmov d7, r2, r3
          vadd.f64 d6, d6, d7
          vmov r0, r1, d6");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __addsf3vfp() {
    #[cfg(armhf)]
    asm!("vadd.f32 s0, s0, s1");
    #[cfg(not(armhf))]
    asm!("vmov s14, r0
          vmov s15, r1
          vadd.f32 s14, s14, s15
          vmov r0, s14");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __divdf3vfp() {
    #[cfg(armhf)]
    asm!("vdiv.f64 d0, d0, d1");
    #[cfg(not(armhf))]
    asm!("vmov d6, r0, r1
          vmov d7, r2, r3
          vdiv.f64 d5, d6, d7
          vmov r0, r1, d5");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __divsf3vfp() {
    #[cfg(armhf)]
    asm!("vdiv.f32 s0, s0, s1");
    #[cfg(not(armhf))]
    asm!("vmov s14, r0
          vmov s15, r1
          vdiv.f32 s13, s14, s15
          vmov r0, s13");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __muldf3vfp() {
    #[cfg(armhf)]
    asm!("vmul.f64 d0, d0, d1");
    #[cfg(not(armhf))]
    asm!("vmov d6, r0, r1
          vmov d7, r2, r3
          vmul.f64 d6, d6, d7
          vmov r0, r1, d6");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __mulsf3vfp() {
    #[cfg(armhf)]
    asm!("vmul.f32 s0, s0, s1");
    #[cfg(not(armhf))]
    asm!("vmov s14, r0
          vmov s15, r1
          vmul.f32 s13, s14, s15
          vmov r0, s13");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __negdf2vfp() {
    #[cfg(armhf)]
    asm!("vneg.f64 d0, d0");
    #[cfg(not(armhf))]
    asm!("eor r1, r1, #-2147483648");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __negsf2vfp() {
    #[cfg(armhf)]
    asm!("vneg.f32 s0, s0");
    #[cfg(not(armhf))]
    asm!("eor r0, r0, #-2147483648");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __subdf3vfp() {
    #[cfg(armhf)]
    asm!("vsub.f64 d0, d0, d1");
    #[cfg(not(armhf))]
    asm!("vmov d6, r0, r1
          vmov d7, r2, r3
          vsub.f64 d6, d6, d7
          vmov r0, r1, d6");
    asm!("bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __subsf3vfp() {
    #[cfg(armhf)]
    asm!("vsub.f32 s0, s0, s1");
    #[cfg(not(armhf))]
    asm!("vmov s14, r0
          vmov s15, r1
          vsub.f32 s14, s14, s15
          vmov r0, s14");
    asm!("bx lr");
    intrinsics::unreachable();
}

// Comparison functions

macro_rules! vfp_cmp {
    ($dfn:ident, $sfn:ident, $cmp1:expr, $cmp2:expr) => {
        #[naked]
        #[cfg_attr(not(test), no_mangle)]
        #[cfg(not(no_vfp64))]
        pub unsafe fn $dfn() {
            asm!(concat!("vmov d6, r0, r1
                vmov d7, r2, r3
                vcmp.f64 d6, d7 
                vmrs apsr_nzcv, fpscr
                mov", $cmp1, " r0, #1
                mov", $cmp2, " r0, #0
                bx lr"));
            intrinsics::unreachable();
        }

        #[naked]
        #[cfg_attr(not(test), no_mangle)]
        pub unsafe fn $sfn() {
            asm!(concat!("vmov s14, r0
                vmov s15, r1
                vcmp.f32 s14, s15
                vmrs apsr_nzcv, fpscr
                mov", $cmp1, " r0, #1
                mov", $cmp2, " r0, #0
                bx lr"));
            intrinsics::unreachable();
        }
    }
}

vfp_cmp!(__eqdf2vfp, __eqsf2vfp, "eq", "ne");
vfp_cmp!(__gedf2vfp, __gesf2vfp, "ge", "lt");
vfp_cmp!(__gtdf2vfp, __gtsf2vfp, "gt", "le");
vfp_cmp!(__ledf2vfp, __lesf2vfp, "ls", "hi");
vfp_cmp!(__ltdf2vfp, __ltsf2vfp, "mi", "pl");
vfp_cmp!(__nedf2vfp, __nesf2vfp, "ne", "eq");

macro_rules! vfp_dcmp {
    ($dfn:ident, $sfn:ident, $cmp:expr) => {
        #[naked]
        #[cfg_attr(not(test), no_mangle)]
        #[cfg(not(no_vfp64))]
        pub unsafe fn $dfn() {
            asm!(concat!("push {r4, lr}
                bl __", $cmp, "df2
                cmp r0, #0 
                b", $cmp, " 1f
                mov r0, #0
                pop {r4, pc}
            1:
                mov r0, #1
                pop {r4, pc}"));
            intrinsics::unreachable();
        }

        #[naked]
        #[cfg_attr(not(test), no_mangle)]
        pub unsafe fn $sfn() {
            asm!(concat!("push {r4, lr}
                bl __", $cmp, "sf2
                cmp r0, #0 
                b", $cmp, " 1f
                mov r0, #0
                pop {r4, pc}
            1:
                mov r0, #1
                pop {r4, pc}"));
            intrinsics::unreachable();
        }
    }
}

vfp_dcmp!(__aeabi_dcmpeq, __aeabi_fcmpeq, "eq");
vfp_dcmp!(__aeabi_dcmplt, __aeabi_fcmplt, "lt");
vfp_dcmp!(__aeabi_dcmple, __aeabi_fcmple, "le");
vfp_dcmp!(__aeabi_dcmpge, __aeabi_fcmpge, "ge");
vfp_dcmp!(__aeabi_dcmpgt, __aeabi_fcmpgt, "gt");

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __unorddf2vfp() {
    asm!("vmov d6, r0, r1
          vmov d7, r2, r3
          vcmp.f64 d6, d7 
          vmrs apsr_nzcv, fpscr
          movvs r0, #1
          movvc r0, #0
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __unordsf2vfp() {
    asm!("vmov s14, r0
          vmov s15, r1
          vcmp.f32 s14, s15
          vmrs apsr_nzcv, fpscr
          movvs r0, #1
          movvc r0, #0
          bx lr");
    intrinsics::unreachable();
}

// Conversion functions

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __extendsfdf2vfp() {
    asm!("vmov s15, r0
          vcvt.f64.f32 d7, s15
          vmov r0, r1, d7
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __fixdfsivfp() {
    asm!("vmov d7, r0, r1
          vcvt.s32.f64 s15, d7
          vmov r0, s15
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __fixsfsivfp() {
    asm!("vmov s15, r0
          vcvt.s32.f32 s15, s15
          vmov r0, s15
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __fixunsdfsivfp() {
    asm!("vmov d7, r0, r1
          vcvt.u32.f64 s15, d7
          vmov r0, s15
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __fixunssfsivfp() {
    asm!("vmov s15, r0
          vcvt.u32.f32 s15, s15
          vmov r0, s15
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __floatsidfvfp() {
    asm!("vmov s15, r0
          vcvt.f64.s32 d7, s15
          vmov r0, r1, d7
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __floatsisfvfp() {
    asm!("vmov s15, r0
          vcvt.f32.s32 s15, s15
          vmov r0, s15
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __floatunssidfvfp() {
    asm!("vmov s15, r0
          vcvt.f64.u32 d7, s15
          vmov r0, r1, d7
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
pub unsafe fn __floatunssisfvfp() {
    asm!("vmov s15, r0
          vcvt.f32.u32 s15, s15
          vmov r0, s15
          bx lr");
    intrinsics::unreachable();
}

#[naked]
#[cfg_attr(not(test), no_mangle)]
#[cfg(not(no_vfp64))]
pub unsafe fn __truncdfsf2vfp() {
    asm!("vmov d7, r0, r1
          vcvt.f32.f64 s15, d7
          vmov r0, s15
          bx lr");
    intrinsics::unreachable();
}

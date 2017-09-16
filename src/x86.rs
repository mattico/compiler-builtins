#![allow(unused_imports)]

use core::intrinsics;

// NOTE These functions are implemented using assembly because they using a custom
// calling convention which can't be implemented using a normal Rust function

// NOTE These functions are never mangled as they are not tested against compiler-rt
// and mangling ___chkstk would break the `jmp ___chkstk` instruction in __alloca

#[cfg(all(windows, target_env = "gnu", not(feature = "mangled-names")))]
#[naked]
#[no_mangle]
pub unsafe fn ___chkstk_ms() {
    asm!("
        push   %ecx
        push   %eax
        cmp    $$0x1000,%eax
        lea    12(%esp),%ecx
        jb     1f
    2:
        sub    $$0x1000,%ecx
        test   %ecx,(%ecx)
        sub    $$0x1000,%eax
        cmp    $$0x1000,%eax
        ja     2b
    1:
        sub    %eax,%ecx
        test   %ecx,(%ecx)
        pop    %eax
        pop    %ecx
        ret");
    intrinsics::unreachable();
}

// FIXME: __alloca should be an alias to __chkstk
#[cfg(all(windows, target_env = "gnu", not(feature = "mangled-names")))]
#[naked]
#[no_mangle]
pub unsafe fn __alloca() {
    asm!("jmp ___chkstk   // Jump to ___chkstk since fallthrough may be unreliable");
    intrinsics::unreachable();
}

#[cfg(all(windows, target_env = "gnu", not(feature = "mangled-names")))]
#[naked]
#[no_mangle]
pub unsafe fn ___chkstk() {
    asm!("
        push   %ecx
        cmp    $$0x1000,%eax
        lea    8(%esp),%ecx     // esp before calling this routine -> ecx
        jb     1f
    2:
        sub    $$0x1000,%ecx
        test   %ecx,(%ecx)
        sub    $$0x1000,%eax
        cmp    $$0x1000,%eax
        ja     2b
    1:
        sub    %eax,%ecx
        test   %ecx,(%ecx)

        lea    4(%esp),%eax     // load pointer to the return address into eax
        mov    %ecx,%esp        // install the new top of stack pointer into esp
        mov    -4(%eax),%ecx    // restore ecx
        push   (%eax)           // push return address onto the stack
        sub    %esp,%eax        // restore the original value in eax
        ret");
    intrinsics::unreachable();
}

/// 64-bit arithmetic left shift
#[cfg(all(target_arch = "x86", not(target_env = "msvc")))]
#[cfg_attr(not(feature = "mangled-names"), no_mangle)]
pub extern "C" fn __ashldi3(input: u64, count: u32) -> u64 {
    let ret;

    unsafe {
        #[cfg(target_feature = "sse2")]
        asm!("
            psllq		$2,		$1	  // shift input by count
            movd		$1,		%eax
            psrlq		$$32,	$1
            movd		$1,		%edx"
            : "=A"(ret)
            : "Y"(input), "Y"(count));

        #[cfg(not(target_feature = "sse2"))]
        asm!("
            testl		$$32,		%ecx	// If count >= 32
            jnz		    1f			        //    goto 1
            shldl		%cl, %eax,	%edx	// left shift high by count
            shll		%cl,		%eax	// left shift low by count
            jmp         2f

        1:	movl		%eax,		%edx	// Move low to high
            xorl		%eax,		%eax	// clear low
            shll		%cl,		%edx	// shift high by count - 32
        2:"
            : "=A"(ret)
            : "{ecx}"(count), "A"(input));
    }

    ret
}

/// 64-bit arithmetic right shift
#[cfg(all(target_arch = "x86", not(target_env = "msvc")))]
#[cfg_attr(not(feature = "mangled-names"), no_mangle)]
pub extern "C" fn __ashrdi3(input: i64, count: u32) -> i64 {
    let ret;

    unsafe {
        // issue
        #[cfg(target_feature = "sse2")]
        asm!("
            psrlq		$3,		    $1	    // unsigned shift input by count
            
            testl		$2,		    $2	    // check the sign-bit of the input
            jns			1f					// early out for positive inputs
            
            // If the input is negative, we need to construct the shifted sign bit
            // to or into the result, as xmm does not have a signed right shift.
            pcmpeqb		%xmm1,		%xmm1	// -1ULL
            psrlq		$$58,		%xmm1	// 0x3f
            pandn		%xmm1,		$3	    // 63 - count
            pcmpeqb		%xmm1,		%xmm1	// -1ULL
            psubq		%xmm1,		$3	    // 64 - count
            psllq		$3,		    %xmm1	// -1 << (64 - count) = leading sign bits
            por			%xmm1,		$1
            
            // Move the result back to the general purpose registers and return
        1:	movd		$1,		    %eax
            psrlq		$$32,	    $1
            movd		$1,		    %edx"
            : "=A"(ret)
            : "Y"(input), "r"(input), "Y"(count)
            : "xmm1");

        // issue
        #[cfg(not(target_feature = "sse2"))]
        asm!("
            movl	  12(%esp),		%ecx	// Load count
            movl	   8(%esp),		%edx	// Load high
            movl	   4(%esp),		%eax	// Load low
            
            testl		$$32,		%ecx	// If count >= 32
            jnz			1f					//    goto 1

            shrdl		%cl, %edx,	%eax	// right shift low by count
            sarl		%cl,		%edx	// right shift high by count
            ret
            
        1:	movl		%edx,		%eax	// Move high to low
            sarl		$$31,		%edx	// clear high
            sarl		%cl,		%eax	// shift low by count - 32"
            : "=A"(ret)
            : "{ecx}"(count), "A"(input));
    }

    ret
}

/// 64-bit logical right shift
#[cfg(all(target_arch = "x86", not(target_env = "msvc")))]
#[cfg_attr(not(feature = "mangled-names"), no_mangle)]
pub extern "C" fn __lshrdi3(input: u64, count: u32) -> u64 {
    let ret;

    unsafe {
        #[cfg(target_feature = "sse2")]
        asm!("
            psrlq		$2,		$1	// shift input by count
            movd		$1,		%eax
            psrlq		$$32,	$1
            movd		$1,		%edx"
            : "=A"(ret)
            : "Y"(input), "Y"(count));

        #[cfg(not(target_feature = "sse2"))]
        asm!("       
            testl		$$32,		%ecx	// If count >= 32
            jnz			1f					//    goto 1

            shrdl		%cl, %edx,	%eax	// right shift low by count
            shrl		%cl,		%edx	// right shift high by count
            ret
            
        1:	movl		%edx,		%eax	// Move high to low
            xorl		%edx,		%edx	// clear high
            shrl		%cl,		%eax	// shift low by count - 32"
            : "=A"(ret)
            : "{ecx}"(count), "A"(input));
    }

    ret
}

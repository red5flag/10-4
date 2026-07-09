// vim: ft=arm

    .arm
    .text
    .global armvfpv2_mmm_f32_4x4_0_21_10
    .type armvfpv2_mmm_f32_4x4_0_21_10, %function

// C tile:

//  s16 s20 s24 s28
//  s17 s21 s25 s29
//  s18 s22 s26 s30
//  s19 s23 s27 s31

// packed A: (2x4) alternating between (s0-s3) and (s4-s7)
// packed B: (2x4) alternating between (s8-s11) and (s12-15)

// all vfp registers in use.

armvfpv2_mmm_f32_4x4_0_21_10:

/*
    pld [r1]
    pld [r1, #8]
    pld [r2]
    pld [r2, #8]
*/

    push        { r4-r12 }               // no lr (we're a leaf), no fp. #24 bytes

    ldr         r8, [sp, #28]
    ldr         r9, [sp, #24]

//  r8=rsc, r9=csc

    vmrs        r6, FPSCR
    bic         r6, r6, #0x00370000
    vmsr        FPSCR, r6

    vpush       { s16-s31 }

// vim: ft=arm

.non_linear:

.non_linear_loop_entry:
    sub     r0, #20

.non_linear_loop:
    add     r0, #20
    ldm     r0, { r2, r3, r4, r5, r6 }

    cmp     r2, #29
    movgt   r2, #29
    cmp     r2, #0
    movlt   r2, #29

    add     pc, pc, r2, LSL#2
    nop     // pc in Rn above is start of the add instruction + 8, hence a nop is needed
            // This is A32 asm, for T32/Thump2 use nop.w and b.w to avoid problems.

        b .done

        b .clear

        b .load_tile

        b .scalar_min

        b .scalar_max

        b .scalar_add

        b .scalar_mul

        b .scalar_sub

        b .scalar_sub_flipped

        b .leaky_relu

        b .per_row_min

        b .per_row_max

        b .per_row_add

        b .per_row_mul

        b .per_row_sub

        b .per_row_sub_flipped

        b .per_col_min

        b .per_col_max

        b .per_col_add

        b .per_col_mul

        b .per_col_sub

        b .per_col_sub_flipped

        b .q_scale

        b .q_shr

        b .q_shl

        b .add_unicast

        b .add_row_col_products

        b .store

        b .add_mat_mul

    b .unsupported

.unsupported:
    mov         r0,     #1
    b           .return

.done:
    mov         r0,     #0
    b           .return



.clear:
    eor         r6, r6
    vmov        s16, r6
    vmov.f32    s17, s16
    vmov.f32    s18, s16
    vmov.f32    s19, s16
    vmov.f32    s20, s16
    vmov.f32    s21, s16
    vmov.f32    s22, s16
    vmov.f32    s23, s16
    vmov.f32    s24, s16
    vmov.f32    s25, s16
    vmov.f32    s26, s16
    vmov.f32    s27, s16
    vmov.f32    s28, s16
    vmov.f32    s29, s16
    vmov.f32    s30, s16
    vmov.f32    s31, s16
    b     .non_linear_loop

.add_mat_mul:
    // r3 <- k, r4 <- a, r5 <- b
    cmp     r3, #0
    beq     .non_linear_loop

    mov     r1, r4 // packed A ptr
    pld     [r3]
    pld     [r5]

    .packed_packed:
    cmp r3, #4
    blt .packed_packed_loop_1

    .packed_packed_loop_4:

    // 1
    vldmia          r1!, { s0, s1 }
    vldmia          r5!, { s8, s9 }

    vmla.f32        s16, s0, s8
    vldmia          r1!, { s2, s3 }
    vmla.f32        s17, s1, s8
    vldmia          r5!, { s10, s11 }
    vmla.f32        s18, s2, s8
    vmla.f32        s19, s3, s8

    vmla.f32        s20, s0, s9
    vmla.f32        s21, s1, s9
    vmla.f32        s22, s2, s9
    vmla.f32        s23, s3, s9

    vldmia          r1!, { s4-s7 }
    vmla.f32        s24, s0, s10
    vmla.f32        s25, s1, s10
    vmla.f32        s26, s2, s10
    vmla.f32        s27, s3, s10

    vldmia          r5!, { s12-s15 }
    vmla.f32        s28, s0, s11
    vmla.f32        s29, s1, s11
    vmla.f32        s30, s2, s11
    vmla.f32        s31, s3, s11

    // 2
    vmla.f32        s16, s4, s12
    vmla.f32        s17, s5, s12
    vmla.f32        s18, s6, s12
    vmla.f32        s19, s7, s12

    vldmia          r1!, { s0-s3 }

    vmla.f32        s20, s4, s13
    vmla.f32        s21, s5, s13
    vmla.f32        s22, s6, s13
    vmla.f32        s23, s7, s13

    vldmia          r5!, { s8-s11 }

    vmla.f32        s24, s4, s14
    vmla.f32        s25, s5, s14
    vmla.f32        s26, s6, s14
    vmla.f32        s27, s7, s14

    vmla.f32        s28, s4, s15
    vmla.f32        s29, s5, s15
    vmla.f32        s30, s6, s15
    vmla.f32        s31, s7, s15

    // 3
    vmla.f32        s16, s0, s8
    vmla.f32        s17, s1, s8
    vmla.f32        s18, s2, s8
    vmla.f32        s19, s3, s8

    vldmia          r1!, { s4-s7 }

    vmla.f32        s20, s0, s9
    vmla.f32        s21, s1, s9
    vmla.f32        s22, s2, s9
    vmla.f32        s23, s3, s9

    vldmia          r5!, { s12-s15 }

    vmla.f32        s24, s0, s10
    vmla.f32        s25, s1, s10
    vmla.f32        s26, s2, s10
    vmla.f32        s27, s3, s10

    pld [r1]

    vmla.f32        s28, s0, s11
    vmla.f32        s29, s1, s11
    vmla.f32        s30, s2, s11
    vmla.f32        s31, s3, s11

    pld [r6]

    // 4
    vmla.f32        s16, s4, s12
    vmla.f32        s17, s5, s12
    vmla.f32        s18, s6, s12
    vmla.f32        s19, s7, s12

    vmla.f32        s20, s4, s13
    vmla.f32        s21, s5, s13
    vmla.f32        s22, s6, s13
    vmla.f32        s23, s7, s13

    vmla.f32        s24, s4, s14
    vmla.f32        s25, s5, s14
    vmla.f32        s26, s6, s14
    vmla.f32        s27, s7, s14

    vmla.f32        s28, s4, s15
    vmla.f32        s29, s5, s15
    vmla.f32        s30, s6, s15
    vmla.f32        s31, s7, s15

    sub r3, r3, #4
    cmp r3, #4
    bge .packed_packed_loop_4

    cmp r3, #0
    beq .non_linear_loop

    .packed_packed_loop_1:

    vldmia          r1!, { s0, s1 }
    vldmia          r5!, { s8, s9 }

    vmla.f32        s16, s0, s8
    vldmia          r1!, { s2, s3 }
    vmla.f32        s17, s1, s8
    vldmia          r5!, { s10, s11 }
    vmla.f32        s18, s2, s8
    vmla.f32        s19, s3, s8

    vmla.f32        s20, s0, s9
    vmla.f32        s21, s1, s9
    vmla.f32        s22, s2, s9
    vmla.f32        s23, s3, s9

    vmla.f32        s24, s0, s10
    vmla.f32        s25, s1, s10
    vmla.f32        s26, s2, s10
    vmla.f32        s27, s3, s10

    vmla.f32        s28, s0, s11
    vmla.f32        s29, s1, s11
    vmla.f32        s30, s2, s11
    vmla.f32        s31, s3, s11

    subs r3, r3, #1
    bne .packed_packed_loop_1

    b .non_linear_loop

.add_unicast:
    
        mov         r8, r3
        
            vldr            s0, [ r8 ]
            vadd.f32        s16, s16, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s17, s17, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s18, s18, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s19, s19, s0
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vldr            s0, [ r8 ]
            vadd.f32        s20, s20, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s21, s21, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s22, s22, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s23, s23, s0
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vldr            s0, [ r8 ]
            vadd.f32        s24, s24, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s25, s25, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s26, s26, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s27, s27, s0
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vldr            s0, [ r8 ]
            vadd.f32        s28, s28, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s29, s29, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s30, s30, s0
            
                add         r8, r8, r4
            
        
            vldr            s0, [ r8 ]
            vadd.f32        s31, s31, s0
            
        
        
    

    b .non_linear_loop

.scalar_min:
    vmov            s0, r3
    
        vcmp.f32        s16, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s16, s0
    
        vcmp.f32        s17, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s17, s0
    
        vcmp.f32        s18, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s18, s0
    
        vcmp.f32        s19, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s19, s0
    
        vcmp.f32        s20, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s20, s0
    
        vcmp.f32        s21, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s21, s0
    
        vcmp.f32        s22, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s22, s0
    
        vcmp.f32        s23, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s23, s0
    
        vcmp.f32        s24, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s24, s0
    
        vcmp.f32        s25, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s25, s0
    
        vcmp.f32        s26, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s26, s0
    
        vcmp.f32        s27, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s27, s0
    
        vcmp.f32        s28, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s28, s0
    
        vcmp.f32        s29, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s29, s0
    
        vcmp.f32        s30, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s30, s0
    
        vcmp.f32        s31, s0
        vmrs            apsr_nzcv, fpscr
        vmovge          s31, s0
    

    b .non_linear_loop

.scalar_max:
    vmov            s0, r3
    
        vcmp.f32        s16, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s16, s0
    
        vcmp.f32        s17, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s17, s0
    
        vcmp.f32        s18, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s18, s0
    
        vcmp.f32        s19, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s19, s0
    
        vcmp.f32        s20, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s20, s0
    
        vcmp.f32        s21, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s21, s0
    
        vcmp.f32        s22, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s22, s0
    
        vcmp.f32        s23, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s23, s0
    
        vcmp.f32        s24, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s24, s0
    
        vcmp.f32        s25, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s25, s0
    
        vcmp.f32        s26, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s26, s0
    
        vcmp.f32        s27, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s27, s0
    
        vcmp.f32        s28, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s28, s0
    
        vcmp.f32        s29, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s29, s0
    
        vcmp.f32        s30, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s30, s0
    
        vcmp.f32        s31, s0
        vmrs            apsr_nzcv, fpscr
        vmovle          s31, s0
    

    b .non_linear_loop

.scalar_add:
    vmov            s0, r3
    
        vadd.f32    s16, s16, s0
    
        vadd.f32    s17, s17, s0
    
        vadd.f32    s18, s18, s0
    
        vadd.f32    s19, s19, s0
    
        vadd.f32    s20, s20, s0
    
        vadd.f32    s21, s21, s0
    
        vadd.f32    s22, s22, s0
    
        vadd.f32    s23, s23, s0
    
        vadd.f32    s24, s24, s0
    
        vadd.f32    s25, s25, s0
    
        vadd.f32    s26, s26, s0
    
        vadd.f32    s27, s27, s0
    
        vadd.f32    s28, s28, s0
    
        vadd.f32    s29, s29, s0
    
        vadd.f32    s30, s30, s0
    
        vadd.f32    s31, s31, s0
    

    b .non_linear_loop

.scalar_mul:
    vmov            s0, r3
    
        vmul.f32    s16, s16, s0
    
        vmul.f32    s17, s17, s0
    
        vmul.f32    s18, s18, s0
    
        vmul.f32    s19, s19, s0
    
        vmul.f32    s20, s20, s0
    
        vmul.f32    s21, s21, s0
    
        vmul.f32    s22, s22, s0
    
        vmul.f32    s23, s23, s0
    
        vmul.f32    s24, s24, s0
    
        vmul.f32    s25, s25, s0
    
        vmul.f32    s26, s26, s0
    
        vmul.f32    s27, s27, s0
    
        vmul.f32    s28, s28, s0
    
        vmul.f32    s29, s29, s0
    
        vmul.f32    s30, s30, s0
    
        vmul.f32    s31, s31, s0
    

    b .non_linear_loop

.scalar_sub:
    vmov            s0, r3
    
        vsub.f32    s16, s0, s16
    
        vsub.f32    s17, s0, s17
    
        vsub.f32    s18, s0, s18
    
        vsub.f32    s19, s0, s19
    
        vsub.f32    s20, s0, s20
    
        vsub.f32    s21, s0, s21
    
        vsub.f32    s22, s0, s22
    
        vsub.f32    s23, s0, s23
    
        vsub.f32    s24, s0, s24
    
        vsub.f32    s25, s0, s25
    
        vsub.f32    s26, s0, s26
    
        vsub.f32    s27, s0, s27
    
        vsub.f32    s28, s0, s28
    
        vsub.f32    s29, s0, s29
    
        vsub.f32    s30, s0, s30
    
        vsub.f32    s31, s0, s31
    

    b .non_linear_loop

.scalar_sub_flipped:
    vmov            s0, r3
    
        vsub.f32    s16, s16, s0
    
        vsub.f32    s17, s17, s0
    
        vsub.f32    s18, s18, s0
    
        vsub.f32    s19, s19, s0
    
        vsub.f32    s20, s20, s0
    
        vsub.f32    s21, s21, s0
    
        vsub.f32    s22, s22, s0
    
        vsub.f32    s23, s23, s0
    
        vsub.f32    s24, s24, s0
    
        vsub.f32    s25, s25, s0
    
        vsub.f32    s26, s26, s0
    
        vsub.f32    s27, s27, s0
    
        vsub.f32    s28, s28, s0
    
        vsub.f32    s29, s29, s0
    
        vsub.f32    s30, s30, s0
    
        vsub.f32    s31, s31, s0
    

    b .non_linear_loop

.leaky_relu:
    vmov            s0, r3
    
        vmul.f32        s1, s0, s16
        vcmp.f32        s16, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s16, s1
    
        vmul.f32        s1, s0, s17
        vcmp.f32        s17, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s17, s1
    
        vmul.f32        s1, s0, s18
        vcmp.f32        s18, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s18, s1
    
        vmul.f32        s1, s0, s19
        vcmp.f32        s19, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s19, s1
    
        vmul.f32        s1, s0, s20
        vcmp.f32        s20, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s20, s1
    
        vmul.f32        s1, s0, s21
        vcmp.f32        s21, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s21, s1
    
        vmul.f32        s1, s0, s22
        vcmp.f32        s22, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s22, s1
    
        vmul.f32        s1, s0, s23
        vcmp.f32        s23, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s23, s1
    
        vmul.f32        s1, s0, s24
        vcmp.f32        s24, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s24, s1
    
        vmul.f32        s1, s0, s25
        vcmp.f32        s25, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s25, s1
    
        vmul.f32        s1, s0, s26
        vcmp.f32        s26, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s26, s1
    
        vmul.f32        s1, s0, s27
        vcmp.f32        s27, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s27, s1
    
        vmul.f32        s1, s0, s28
        vcmp.f32        s28, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s28, s1
    
        vmul.f32        s1, s0, s29
        vcmp.f32        s29, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s29, s1
    
        vmul.f32        s1, s0, s30
        vcmp.f32        s30, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s30, s1
    
        vmul.f32        s1, s0, s31
        vcmp.f32        s31, #0
        vmrs            apsr_nzcv, fpscr
        vmovlt          s31, s1
    
    b .non_linear_loop

.per_row_min:
    vldm    r3, {s0, s1, s2, s3}
    
        
            
            vcmp.f32        s16, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s16, s0
        
            
            vcmp.f32        s20, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s20, s0
        
            
            vcmp.f32        s24, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s24, s0
        
            
            vcmp.f32        s28, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s28, s0
        
    
        
            
            vcmp.f32        s17, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s17, s1
        
            
            vcmp.f32        s21, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s21, s1
        
            
            vcmp.f32        s25, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s25, s1
        
            
            vcmp.f32        s29, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s29, s1
        
    
        
            
            vcmp.f32        s18, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s18, s2
        
            
            vcmp.f32        s22, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s22, s2
        
            
            vcmp.f32        s26, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s26, s2
        
            
            vcmp.f32        s30, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s30, s2
        
    
        
            
            vcmp.f32        s19, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s19, s3
        
            
            vcmp.f32        s23, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s23, s3
        
            
            vcmp.f32        s27, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s27, s3
        
            
            vcmp.f32        s31, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s31, s3
        
    

    b .non_linear_loop

.per_row_max:
    vldm    r3, {s0, s1, s2, s3}
    
        
            
            vcmp.f32        s16, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s16, s0
        
            
            vcmp.f32        s20, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s20, s0
        
            
            vcmp.f32        s24, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s24, s0
        
            
            vcmp.f32        s28, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s28, s0
        
    
        
            
            vcmp.f32        s17, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s17, s1
        
            
            vcmp.f32        s21, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s21, s1
        
            
            vcmp.f32        s25, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s25, s1
        
            
            vcmp.f32        s29, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s29, s1
        
    
        
            
            vcmp.f32        s18, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s18, s2
        
            
            vcmp.f32        s22, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s22, s2
        
            
            vcmp.f32        s26, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s26, s2
        
            
            vcmp.f32        s30, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s30, s2
        
    
        
            
            vcmp.f32        s19, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s19, s3
        
            
            vcmp.f32        s23, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s23, s3
        
            
            vcmp.f32        s27, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s27, s3
        
            
            vcmp.f32        s31, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s31, s3
        
    

    b .non_linear_loop

.per_row_add:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vadd.f32    s16, s16, s0
        
            vadd.f32    s20, s20, s0
        
            vadd.f32    s24, s24, s0
        
            vadd.f32    s28, s28, s0
        
    
        
            vadd.f32    s17, s17, s1
        
            vadd.f32    s21, s21, s1
        
            vadd.f32    s25, s25, s1
        
            vadd.f32    s29, s29, s1
        
    
        
            vadd.f32    s18, s18, s2
        
            vadd.f32    s22, s22, s2
        
            vadd.f32    s26, s26, s2
        
            vadd.f32    s30, s30, s2
        
    
        
            vadd.f32    s19, s19, s3
        
            vadd.f32    s23, s23, s3
        
            vadd.f32    s27, s27, s3
        
            vadd.f32    s31, s31, s3
        
    

    b .non_linear_loop

.per_row_mul:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vmul.f32    s16, s16, s0
        
            vmul.f32    s20, s20, s0
        
            vmul.f32    s24, s24, s0
        
            vmul.f32    s28, s28, s0
        
    
        
            vmul.f32    s17, s17, s1
        
            vmul.f32    s21, s21, s1
        
            vmul.f32    s25, s25, s1
        
            vmul.f32    s29, s29, s1
        
    
        
            vmul.f32    s18, s18, s2
        
            vmul.f32    s22, s22, s2
        
            vmul.f32    s26, s26, s2
        
            vmul.f32    s30, s30, s2
        
    
        
            vmul.f32    s19, s19, s3
        
            vmul.f32    s23, s23, s3
        
            vmul.f32    s27, s27, s3
        
            vmul.f32    s31, s31, s3
        
    

    b .non_linear_loop

.per_row_sub:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vsub.f32    s16, s0, s16
        
            vsub.f32    s20, s0, s20
        
            vsub.f32    s24, s0, s24
        
            vsub.f32    s28, s0, s28
        
    
        
            vsub.f32    s17, s1, s17
        
            vsub.f32    s21, s1, s21
        
            vsub.f32    s25, s1, s25
        
            vsub.f32    s29, s1, s29
        
    
        
            vsub.f32    s18, s2, s18
        
            vsub.f32    s22, s2, s22
        
            vsub.f32    s26, s2, s26
        
            vsub.f32    s30, s2, s30
        
    
        
            vsub.f32    s19, s3, s19
        
            vsub.f32    s23, s3, s23
        
            vsub.f32    s27, s3, s27
        
            vsub.f32    s31, s3, s31
        
    

    b .non_linear_loop

.per_row_sub_flipped:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vsub.f32    s16, s16, s0
        
            vsub.f32    s20, s20, s0
        
            vsub.f32    s24, s24, s0
        
            vsub.f32    s28, s28, s0
        
    
        
            vsub.f32    s17, s17, s1
        
            vsub.f32    s21, s21, s1
        
            vsub.f32    s25, s25, s1
        
            vsub.f32    s29, s29, s1
        
    
        
            vsub.f32    s18, s18, s2
        
            vsub.f32    s22, s22, s2
        
            vsub.f32    s26, s26, s2
        
            vsub.f32    s30, s30, s2
        
    
        
            vsub.f32    s19, s19, s3
        
            vsub.f32    s23, s23, s3
        
            vsub.f32    s27, s27, s3
        
            vsub.f32    s31, s31, s3
        
    

    b .non_linear_loop

.per_col_min:
    vldm    r3, {s0, s1, s2, s3}
    
        
            
            vcmp.f32        s16, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s16, s0
        
            
            vcmp.f32        s20, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s20, s1
        
            
            vcmp.f32        s24, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s24, s2
        
            
            vcmp.f32        s28, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s28, s3
        
    
        
            
            vcmp.f32        s17, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s17, s0
        
            
            vcmp.f32        s21, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s21, s1
        
            
            vcmp.f32        s25, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s25, s2
        
            
            vcmp.f32        s29, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s29, s3
        
    
        
            
            vcmp.f32        s18, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s18, s0
        
            
            vcmp.f32        s22, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s22, s1
        
            
            vcmp.f32        s26, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s26, s2
        
            
            vcmp.f32        s30, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s30, s3
        
    
        
            
            vcmp.f32        s19, s0
            vmrs            apsr_nzcv, fpscr
            vmovge          s19, s0
        
            
            vcmp.f32        s23, s1
            vmrs            apsr_nzcv, fpscr
            vmovge          s23, s1
        
            
            vcmp.f32        s27, s2
            vmrs            apsr_nzcv, fpscr
            vmovge          s27, s2
        
            
            vcmp.f32        s31, s3
            vmrs            apsr_nzcv, fpscr
            vmovge          s31, s3
        
    

    b .non_linear_loop

.per_col_max:
    vldm    r3, {s0, s1, s2, s3}
    
        
            
            vcmp.f32        s16, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s16, s0
        
            
            vcmp.f32        s20, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s20, s1
        
            
            vcmp.f32        s24, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s24, s2
        
            
            vcmp.f32        s28, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s28, s3
        
    
        
            
            vcmp.f32        s17, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s17, s0
        
            
            vcmp.f32        s21, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s21, s1
        
            
            vcmp.f32        s25, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s25, s2
        
            
            vcmp.f32        s29, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s29, s3
        
    
        
            
            vcmp.f32        s18, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s18, s0
        
            
            vcmp.f32        s22, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s22, s1
        
            
            vcmp.f32        s26, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s26, s2
        
            
            vcmp.f32        s30, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s30, s3
        
    
        
            
            vcmp.f32        s19, s0
            vmrs            apsr_nzcv, fpscr
            vmovlt          s19, s0
        
            
            vcmp.f32        s23, s1
            vmrs            apsr_nzcv, fpscr
            vmovlt          s23, s1
        
            
            vcmp.f32        s27, s2
            vmrs            apsr_nzcv, fpscr
            vmovlt          s27, s2
        
            
            vcmp.f32        s31, s3
            vmrs            apsr_nzcv, fpscr
            vmovlt          s31, s3
        
    

    b .non_linear_loop

.per_col_add:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vadd.f32    s16, s16, s0
        
            vadd.f32    s20, s20, s1
        
            vadd.f32    s24, s24, s2
        
            vadd.f32    s28, s28, s3
        
    
        
            vadd.f32    s17, s17, s0
        
            vadd.f32    s21, s21, s1
        
            vadd.f32    s25, s25, s2
        
            vadd.f32    s29, s29, s3
        
    
        
            vadd.f32    s18, s18, s0
        
            vadd.f32    s22, s22, s1
        
            vadd.f32    s26, s26, s2
        
            vadd.f32    s30, s30, s3
        
    
        
            vadd.f32    s19, s19, s0
        
            vadd.f32    s23, s23, s1
        
            vadd.f32    s27, s27, s2
        
            vadd.f32    s31, s31, s3
        
    

    b .non_linear_loop

.per_col_mul:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vmul.f32    s16, s16, s0
        
            vmul.f32    s20, s20, s1
        
            vmul.f32    s24, s24, s2
        
            vmul.f32    s28, s28, s3
        
    
        
            vmul.f32    s17, s17, s0
        
            vmul.f32    s21, s21, s1
        
            vmul.f32    s25, s25, s2
        
            vmul.f32    s29, s29, s3
        
    
        
            vmul.f32    s18, s18, s0
        
            vmul.f32    s22, s22, s1
        
            vmul.f32    s26, s26, s2
        
            vmul.f32    s30, s30, s3
        
    
        
            vmul.f32    s19, s19, s0
        
            vmul.f32    s23, s23, s1
        
            vmul.f32    s27, s27, s2
        
            vmul.f32    s31, s31, s3
        
    

    b .non_linear_loop

.per_col_sub:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vsub.f32    s16, s0, s16
        
            vsub.f32    s20, s1, s20
        
            vsub.f32    s24, s2, s24
        
            vsub.f32    s28, s3, s28
        
    
        
            vsub.f32    s17, s0, s17
        
            vsub.f32    s21, s1, s21
        
            vsub.f32    s25, s2, s25
        
            vsub.f32    s29, s3, s29
        
    
        
            vsub.f32    s18, s0, s18
        
            vsub.f32    s22, s1, s22
        
            vsub.f32    s26, s2, s26
        
            vsub.f32    s30, s3, s30
        
    
        
            vsub.f32    s19, s0, s19
        
            vsub.f32    s23, s1, s23
        
            vsub.f32    s27, s2, s27
        
            vsub.f32    s31, s3, s31
        
    

    b .non_linear_loop

.per_col_sub_flipped:
    vldm    r3, {s0, s1, s2, s3}
    
        
            vsub.f32    s16, s16, s0
        
            vsub.f32    s20, s20, s1
        
            vsub.f32    s24, s24, s2
        
            vsub.f32    s28, s28, s3
        
    
        
            vsub.f32    s17, s17, s0
        
            vsub.f32    s21, s21, s1
        
            vsub.f32    s25, s25, s2
        
            vsub.f32    s29, s29, s3
        
    
        
            vsub.f32    s18, s18, s0
        
            vsub.f32    s22, s22, s1
        
            vsub.f32    s26, s26, s2
        
            vsub.f32    s30, s30, s3
        
    
        
            vsub.f32    s19, s19, s0
        
            vsub.f32    s23, s23, s1
        
            vsub.f32    s27, s27, s2
        
            vsub.f32    s31, s31, s3
        
    

    b .non_linear_loop

.add_row_col_products:
    vldmia          r3!, { s0, s1 }
    vldmia          r4!, { s8, s9 }

    vmla.f32        s16, s0, s8
    vldmia          r3!, { s2, s3 }
    vmla.f32        s17, s1, s8
    vldmia          r4!, { s10, s11 }
    vmla.f32        s18, s2, s8
    vmla.f32        s19, s3, s8

    vmla.f32        s20, s0, s9
    vmla.f32        s21, s1, s9
    vmla.f32        s22, s2, s9
    vmla.f32        s23, s3, s9

    vmla.f32        s24, s0, s10
    vmla.f32        s25, s1, s10
    vmla.f32        s26, s2, s10
    vmla.f32        s27, s3, s10

    vmla.f32        s28, s0, s11
    vmla.f32        s29, s1, s11
    vmla.f32        s30, s2, s11
    vmla.f32        s31, s3, s11

    b .non_linear_loop

.store:
    
        mov         r8, r3
        
            fsts        s16, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s17, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s18, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s19, [ r8 ]
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            fsts        s20, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s21, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s22, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s23, [ r8 ]
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            fsts        s24, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s25, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s26, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s27, [ r8 ]
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            fsts        s28, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s29, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s30, [ r8 ]
            
                add         r8, r8, r4
            
        
            fsts        s31, [ r8 ]
            
        
        
    

    mov         r0,     #0
    b   .return

.load_tile:
    vldmia       r3!, { s16-s31 }
    b .non_linear_loop

.q_scale:
.q_shl:
.q_shr:
    b   .unsupported

.return:
    vpop        { s16-s31 }
    pop         { r4-r12 }

    bx          lr


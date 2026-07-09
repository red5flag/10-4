// vim: ft=arm

    .arm
    .text
    .global armv7neon_mmm_f32_8x1_generic_0_21_10
    .type armv7neon_mmm_f32_8x1_generic_0_21_10, %function

armv7neon_mmm_f32_8x1_generic_0_21_10:

    pld     [r0]
    push    { r4-r12 }
    vpush   { q4-q7 }

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

    veor    q4, q4, q4

    veor    q5, q5, q5

    veor    q6, q6, q6

    veor    q7, q7, q7

    veor    q8, q8, q8

    veor    q9, q9, q9

    veor    q10, q10, q10

    veor    q11, q11, q11

    veor    q12, q12, q12

    veor    q13, q13, q13

    veor    q14, q14, q14

    veor    q15, q15, q15

    b           .non_linear_loop


.add_mat_mul:

    cmp     r3, #0
    beq     .non_linear_loop

    mov     r1, r4 // packed A ptr
    pld     [r3]
    pld     [r5]

    pld     [r1, #128]
    pld     [r1, #192]
    pld     [r1, #256]
    pld     [r1, #320]
    pld     [r1, #384]
    pld     [r1, #448]
    pld     [r1, #512]

.packed_packed_loop_1:
    pld     [r5]                           // packed B ptr

    vldmia          r1!, { q0-q1 }
    vldmia          r5!, { s30 }

    vmla.f32        q8, q0, d15[0]
    vmla.f32        q9, q1, d15[0]

    subs            r3, r3, #1
    bne .packed_packed_loop_1

    b   .non_linear_loop

// vim: ft=arm

// vim: ft=arm

.scalar_min:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmin.f32    q8, q0, q8
        
            vmin.f32    q9, q0, q9
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_max:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmax.f32    q8, q0, q8
        
            vmax.f32    q9, q0, q9
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_mul:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmul.f32    q8, q0, q8
        
            vmul.f32    q9, q0, q9
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_add:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vadd.f32    q8, q0, q8
        
            vadd.f32    q9, q0, q9
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_sub:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vsub.f32    q8, q0, q8
        
            vsub.f32    q9, q0, q9
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_sub_flipped:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vsub.f32    q8, q8, q0
        
            vsub.f32    q9, q9, q0
        
    
    b .non_linear_loop


.leaky_relu:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        vmul.f32    q2, q8, q0
        vcgt.f32    q1, q8, 0
        vbsl        q1, q8, q2
        vmov        q8, q1
    
        vmul.f32    q2, q9, q0
        vcgt.f32    q1, q9, 0
        vbsl        q1, q9, q2
        vmov        q9, q1
    
    b .non_linear_loop

.q_shl:
.q_shr:
.q_scale:
    b .unsupported

// vim: ft=arm

// vim: ft=arm

.per_row_min:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmin.f32 q8, q0, q8
    
        
        vmin.f32 q9, q1, q9
    


b           .non_linear_loop

// vim: ft=arm

.per_row_max:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmax.f32 q8, q0, q8
    
        
        vmax.f32 q9, q1, q9
    


b           .non_linear_loop

// vim: ft=arm

.per_row_mul:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmul.f32 q8, q0, q8
    
        
        vmul.f32 q9, q1, q9
    


b           .non_linear_loop

// vim: ft=arm

.per_row_add:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vadd.f32 q8, q0, q8
    
        
        vadd.f32 q9, q1, q9
    


b           .non_linear_loop

// vim: ft=arm

.per_row_sub:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vsub.f32 q8, q0, q8
    
        
        vsub.f32 q9, q1, q9
    


b           .non_linear_loop

// vim: ft=arm

.per_row_sub_flipped:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vsub.f32 q8, q8, q0
    
        
        vsub.f32 q9, q9, q1
    


b           .non_linear_loop



// vim: ft=arm

// vim: ft=arm

.per_col_min:








    vld1.f32        d0[0], [ r3 ]




    vdup.f32 q3, d0[0]
    
        
        
            vmin.f32 q8, q3, q8
        
    
        
        
            vmin.f32 q9, q3, q9
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_max:








    vld1.f32        d0[0], [ r3 ]




    vdup.f32 q3, d0[0]
    
        
        
            vmax.f32 q8, q3, q8
        
    
        
        
            vmax.f32 q9, q3, q9
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_mul:








    vld1.f32        d0[0], [ r3 ]




    vdup.f32 q3, d0[0]
    
        
        
            vmul.f32 q8, q3, q8
        
    
        
        
            vmul.f32 q9, q3, q9
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_add:








    vld1.f32        d0[0], [ r3 ]




    vdup.f32 q3, d0[0]
    
        
        
            vadd.f32 q8, q3, q8
        
    
        
        
            vadd.f32 q9, q3, q9
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_sub:








    vld1.f32        d0[0], [ r3 ]




    vdup.f32 q3, d0[0]
    
        
        
            vsub.f32 q8, q3, q8
        
    
        
        
            vsub.f32 q9, q3, q9
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_sub_flipped:








    vld1.f32        d0[0], [ r3 ]




    vdup.f32 q3, d0[0]
    
        
        
            vsub.f32 q8, q8, q3
        
    
        
        
            vsub.f32 q9, q9, q3
        
    


    b .non_linear_loop




.add_unicast:
    
        vld1.f32    d0[0], [ r3 ], r4
        vld1.f32    d0[1], [ r3 ], r4
    
        vld1.f32    d1[0], [ r3 ], r4
        vld1.f32    d1[1], [ r3 ], r4
    
        vld1.f32    d2[0], [ r3 ], r4
        vld1.f32    d2[1], [ r3 ], r4
    
        vld1.f32    d3[0], [ r3 ], r4
        vld1.f32    d3[1], [ r3 ], r4
    
        vld1.f32    d4[0], [ r3 ], r4
        vld1.f32    d4[1], [ r3 ], r4
    
        vld1.f32    d5[0], [ r3 ], r4
        vld1.f32    d5[1], [ r3 ], r4
    
        vld1.f32    d6[0], [ r3 ], r4
        vld1.f32    d6[1], [ r3 ], r4
    
        vld1.f32    d7[0], [ r3 ], r4
        vld1.f32    d7[1], [ r3 ], r4
    
        vld1.f32    d8[0], [ r3 ], r4
        vld1.f32    d8[1], [ r3 ], r4
    
        vld1.f32    d9[0], [ r3 ], r4
        vld1.f32    d9[1], [ r3 ], r4
    
        vld1.f32    d10[0], [ r3 ], r4
        vld1.f32    d10[1], [ r3 ], r4
    
        vld1.f32    d11[0], [ r3 ], r4
        vld1.f32    d11[1], [ r3 ], r4
    
        vld1.f32    d12[0], [ r3 ], r4
        vld1.f32    d12[1], [ r3 ], r4
    
        vld1.f32    d13[0], [ r3 ], r4
        vld1.f32    d13[1], [ r3 ], r4
    
        vld1.f32    d14[0], [ r3 ], r4
        vld1.f32    d14[1], [ r3 ], r4
    
        vld1.f32    d15[0], [ r3 ], r4
        vld1.f32    d15[1], [ r3 ], r4
    
    
        vadd.f32 q8, q8, q0
    
        vadd.f32 q9, q9, q1
    
        vadd.f32 q10, q10, q2
    
        vadd.f32 q11, q11, q3
    
        vadd.f32 q12, q12, q4
    
        vadd.f32 q13, q13, q5
    
        vadd.f32 q14, q14, q6
    
        vadd.f32 q15, q15, q7
    

    b .non_linear_loop

.add_row_col_products:
    vld1.f32        d0[0], [ r4 ]
    vldmia          r3!, { q4-q5 }

    vmla.f32        q8, q4, d0[0]
    vmla.f32        q9, q5, d0[0]

    b .non_linear_loop

.store:
    // r3, r4 <- ptr, rsc
    cmp     r4, #4
    bne     .store_generic

    vst1.f64    {d16-d19}, [r3]!

    b .non_linear_loop

.store_generic:

    
        vst1.f32    d16[0], [r3], r4
        vst1.f32    d16[1], [r3], r4
    
        vst1.f32    d17[0], [r3], r4
        vst1.f32    d17[1], [r3], r4
    
        vst1.f32    d18[0], [r3], r4
        vst1.f32    d18[1], [r3], r4
    
        vst1.f32    d19[0], [r3], r4
        vst1.f32    d19[1], [r3], r4
    

    b .non_linear_loop

.load_tile:
    vldmia          r3!, { q8-q15 }
    b .non_linear_loop

.return:
    vpop        { q4-q7 }
    pop         { r4-r12 }

    bx          lr


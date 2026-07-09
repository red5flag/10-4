// vim: ft=arm

    .arm
    .text
    .global armv7neon_mmm_f32_8x6_cortexa7_0_21_10
    .type armv7neon_mmm_f32_8x6_cortexa7_0_21_10, %function

armv7neon_mmm_f32_8x6_cortexa7_0_21_10:

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

    .packed_packed_loop_1:


    vldr            d0, [r1]
    vldr            d1, [r1, #8]
    vldr            d2, [r1, #16]
    vldr            d3, [r1, #24]
    vldr            d4, [r5]
    vldr            d5, [r5, #8]
    vldr            d6, [r5, #16]



    pld             [r1, #512]
    pld             [r5, #512]


    vmla.f32        q4, q0, d4[0]
    vmla.f32        q5, q1, d4[0]

    vmla.f32        q6, q0, d4[1]
    vmla.f32        q7, q1, d4[1]

    vmla.f32        q8, q0, d5[0]
    vmla.f32        q9, q1, d5[0]

    vmla.f32        q10, q0, d5[1]
    vmla.f32        q11, q1, d5[1]

    vmla.f32        q12, q0, d6[0]
    vmla.f32        q13, q1, d6[0]

    vmla.f32        q14, q0, d6[1]
    vmla.f32        q15, q1, d6[1]


    add             r1, #32
    add             r5, #24


    subs r3, r3, #1
    bne .packed_packed_loop_1
    b   .non_linear_loop

// vim: ft=arm

// vim: ft=arm

.scalar_min:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmin.f32    q4, q0, q4
        
            vmin.f32    q5, q0, q5
        
            vmin.f32    q6, q0, q6
        
            vmin.f32    q7, q0, q7
        
            vmin.f32    q8, q0, q8
        
            vmin.f32    q9, q0, q9
        
            vmin.f32    q10, q0, q10
        
            vmin.f32    q11, q0, q11
        
            vmin.f32    q12, q0, q12
        
            vmin.f32    q13, q0, q13
        
            vmin.f32    q14, q0, q14
        
            vmin.f32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_max:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmax.f32    q4, q0, q4
        
            vmax.f32    q5, q0, q5
        
            vmax.f32    q6, q0, q6
        
            vmax.f32    q7, q0, q7
        
            vmax.f32    q8, q0, q8
        
            vmax.f32    q9, q0, q9
        
            vmax.f32    q10, q0, q10
        
            vmax.f32    q11, q0, q11
        
            vmax.f32    q12, q0, q12
        
            vmax.f32    q13, q0, q13
        
            vmax.f32    q14, q0, q14
        
            vmax.f32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_mul:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmul.f32    q4, q0, q4
        
            vmul.f32    q5, q0, q5
        
            vmul.f32    q6, q0, q6
        
            vmul.f32    q7, q0, q7
        
            vmul.f32    q8, q0, q8
        
            vmul.f32    q9, q0, q9
        
            vmul.f32    q10, q0, q10
        
            vmul.f32    q11, q0, q11
        
            vmul.f32    q12, q0, q12
        
            vmul.f32    q13, q0, q13
        
            vmul.f32    q14, q0, q14
        
            vmul.f32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_add:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vadd.f32    q4, q0, q4
        
            vadd.f32    q5, q0, q5
        
            vadd.f32    q6, q0, q6
        
            vadd.f32    q7, q0, q7
        
            vadd.f32    q8, q0, q8
        
            vadd.f32    q9, q0, q9
        
            vadd.f32    q10, q0, q10
        
            vadd.f32    q11, q0, q11
        
            vadd.f32    q12, q0, q12
        
            vadd.f32    q13, q0, q13
        
            vadd.f32    q14, q0, q14
        
            vadd.f32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_sub:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vsub.f32    q4, q0, q4
        
            vsub.f32    q5, q0, q5
        
            vsub.f32    q6, q0, q6
        
            vsub.f32    q7, q0, q7
        
            vsub.f32    q8, q0, q8
        
            vsub.f32    q9, q0, q9
        
            vsub.f32    q10, q0, q10
        
            vsub.f32    q11, q0, q11
        
            vsub.f32    q12, q0, q12
        
            vsub.f32    q13, q0, q13
        
            vsub.f32    q14, q0, q14
        
            vsub.f32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_sub_flipped:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vsub.f32    q4, q4, q0
        
            vsub.f32    q5, q5, q0
        
            vsub.f32    q6, q6, q0
        
            vsub.f32    q7, q7, q0
        
            vsub.f32    q8, q8, q0
        
            vsub.f32    q9, q9, q0
        
            vsub.f32    q10, q10, q0
        
            vsub.f32    q11, q11, q0
        
            vsub.f32    q12, q12, q0
        
            vsub.f32    q13, q13, q0
        
            vsub.f32    q14, q14, q0
        
            vsub.f32    q15, q15, q0
        
    
    b .non_linear_loop


.leaky_relu:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        vmul.f32    q2, q4, q0
        vcgt.f32    q1, q4, 0
        vbsl        q1, q4, q2
        vmov        q4, q1
    
        vmul.f32    q2, q5, q0
        vcgt.f32    q1, q5, 0
        vbsl        q1, q5, q2
        vmov        q5, q1
    
        vmul.f32    q2, q6, q0
        vcgt.f32    q1, q6, 0
        vbsl        q1, q6, q2
        vmov        q6, q1
    
        vmul.f32    q2, q7, q0
        vcgt.f32    q1, q7, 0
        vbsl        q1, q7, q2
        vmov        q7, q1
    
        vmul.f32    q2, q8, q0
        vcgt.f32    q1, q8, 0
        vbsl        q1, q8, q2
        vmov        q8, q1
    
        vmul.f32    q2, q9, q0
        vcgt.f32    q1, q9, 0
        vbsl        q1, q9, q2
        vmov        q9, q1
    
        vmul.f32    q2, q10, q0
        vcgt.f32    q1, q10, 0
        vbsl        q1, q10, q2
        vmov        q10, q1
    
        vmul.f32    q2, q11, q0
        vcgt.f32    q1, q11, 0
        vbsl        q1, q11, q2
        vmov        q11, q1
    
        vmul.f32    q2, q12, q0
        vcgt.f32    q1, q12, 0
        vbsl        q1, q12, q2
        vmov        q12, q1
    
        vmul.f32    q2, q13, q0
        vcgt.f32    q1, q13, 0
        vbsl        q1, q13, q2
        vmov        q13, q1
    
        vmul.f32    q2, q14, q0
        vcgt.f32    q1, q14, 0
        vbsl        q1, q14, q2
        vmov        q14, q1
    
        vmul.f32    q2, q15, q0
        vcgt.f32    q1, q15, 0
        vbsl        q1, q15, q2
        vmov        q15, q1
    
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



    
        
        vmin.f32 q4, q0, q4
    
        
        vmin.f32 q5, q1, q5
    
        
        vmin.f32 q6, q0, q6
    
        
        vmin.f32 q7, q1, q7
    
        
        vmin.f32 q8, q0, q8
    
        
        vmin.f32 q9, q1, q9
    
        
        vmin.f32 q10, q0, q10
    
        
        vmin.f32 q11, q1, q11
    
        
        vmin.f32 q12, q0, q12
    
        
        vmin.f32 q13, q1, q13
    
        
        vmin.f32 q14, q0, q14
    
        
        vmin.f32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_max:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmax.f32 q4, q0, q4
    
        
        vmax.f32 q5, q1, q5
    
        
        vmax.f32 q6, q0, q6
    
        
        vmax.f32 q7, q1, q7
    
        
        vmax.f32 q8, q0, q8
    
        
        vmax.f32 q9, q1, q9
    
        
        vmax.f32 q10, q0, q10
    
        
        vmax.f32 q11, q1, q11
    
        
        vmax.f32 q12, q0, q12
    
        
        vmax.f32 q13, q1, q13
    
        
        vmax.f32 q14, q0, q14
    
        
        vmax.f32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_mul:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmul.f32 q4, q0, q4
    
        
        vmul.f32 q5, q1, q5
    
        
        vmul.f32 q6, q0, q6
    
        
        vmul.f32 q7, q1, q7
    
        
        vmul.f32 q8, q0, q8
    
        
        vmul.f32 q9, q1, q9
    
        
        vmul.f32 q10, q0, q10
    
        
        vmul.f32 q11, q1, q11
    
        
        vmul.f32 q12, q0, q12
    
        
        vmul.f32 q13, q1, q13
    
        
        vmul.f32 q14, q0, q14
    
        
        vmul.f32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_add:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vadd.f32 q4, q0, q4
    
        
        vadd.f32 q5, q1, q5
    
        
        vadd.f32 q6, q0, q6
    
        
        vadd.f32 q7, q1, q7
    
        
        vadd.f32 q8, q0, q8
    
        
        vadd.f32 q9, q1, q9
    
        
        vadd.f32 q10, q0, q10
    
        
        vadd.f32 q11, q1, q11
    
        
        vadd.f32 q12, q0, q12
    
        
        vadd.f32 q13, q1, q13
    
        
        vadd.f32 q14, q0, q14
    
        
        vadd.f32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_sub:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vsub.f32 q4, q0, q4
    
        
        vsub.f32 q5, q1, q5
    
        
        vsub.f32 q6, q0, q6
    
        
        vsub.f32 q7, q1, q7
    
        
        vsub.f32 q8, q0, q8
    
        
        vsub.f32 q9, q1, q9
    
        
        vsub.f32 q10, q0, q10
    
        
        vsub.f32 q11, q1, q11
    
        
        vsub.f32 q12, q0, q12
    
        
        vsub.f32 q13, q1, q13
    
        
        vsub.f32 q14, q0, q14
    
        
        vsub.f32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_sub_flipped:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vsub.f32 q4, q4, q0
    
        
        vsub.f32 q5, q5, q1
    
        
        vsub.f32 q6, q6, q0
    
        
        vsub.f32 q7, q7, q1
    
        
        vsub.f32 q8, q8, q0
    
        
        vsub.f32 q9, q9, q1
    
        
        vsub.f32 q10, q10, q0
    
        
        vsub.f32 q11, q11, q1
    
        
        vsub.f32 q12, q12, q0
    
        
        vsub.f32 q13, q13, q1
    
        
        vsub.f32 q14, q14, q0
    
        
        vsub.f32 q15, q15, q1
    


b           .non_linear_loop



// vim: ft=arm

// vim: ft=arm

.per_col_min:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    
        vldmia      r3!, { d2 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vmin.f32 q4, q3, q4
        
    
        
        
            vmin.f32 q5, q3, q5
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vmin.f32 q6, q3, q6
        
    
        
        
            vmin.f32 q7, q3, q7
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vmin.f32 q8, q3, q8
        
    
        
        
            vmin.f32 q9, q3, q9
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vmin.f32 q10, q3, q10
        
    
        
        
            vmin.f32 q11, q3, q11
        
    

    vdup.f32 q3, d2[0]
    
        
        
            vmin.f32 q12, q3, q12
        
    
        
        
            vmin.f32 q13, q3, q13
        
    

    vdup.f32 q3, d2[1]
    
        
        
            vmin.f32 q14, q3, q14
        
    
        
        
            vmin.f32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_max:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    
        vldmia      r3!, { d2 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vmax.f32 q4, q3, q4
        
    
        
        
            vmax.f32 q5, q3, q5
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vmax.f32 q6, q3, q6
        
    
        
        
            vmax.f32 q7, q3, q7
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vmax.f32 q8, q3, q8
        
    
        
        
            vmax.f32 q9, q3, q9
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vmax.f32 q10, q3, q10
        
    
        
        
            vmax.f32 q11, q3, q11
        
    

    vdup.f32 q3, d2[0]
    
        
        
            vmax.f32 q12, q3, q12
        
    
        
        
            vmax.f32 q13, q3, q13
        
    

    vdup.f32 q3, d2[1]
    
        
        
            vmax.f32 q14, q3, q14
        
    
        
        
            vmax.f32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_mul:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    
        vldmia      r3!, { d2 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vmul.f32 q4, q3, q4
        
    
        
        
            vmul.f32 q5, q3, q5
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vmul.f32 q6, q3, q6
        
    
        
        
            vmul.f32 q7, q3, q7
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vmul.f32 q8, q3, q8
        
    
        
        
            vmul.f32 q9, q3, q9
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vmul.f32 q10, q3, q10
        
    
        
        
            vmul.f32 q11, q3, q11
        
    

    vdup.f32 q3, d2[0]
    
        
        
            vmul.f32 q12, q3, q12
        
    
        
        
            vmul.f32 q13, q3, q13
        
    

    vdup.f32 q3, d2[1]
    
        
        
            vmul.f32 q14, q3, q14
        
    
        
        
            vmul.f32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_add:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    
        vldmia      r3!, { d2 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vadd.f32 q4, q3, q4
        
    
        
        
            vadd.f32 q5, q3, q5
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vadd.f32 q6, q3, q6
        
    
        
        
            vadd.f32 q7, q3, q7
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vadd.f32 q8, q3, q8
        
    
        
        
            vadd.f32 q9, q3, q9
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vadd.f32 q10, q3, q10
        
    
        
        
            vadd.f32 q11, q3, q11
        
    

    vdup.f32 q3, d2[0]
    
        
        
            vadd.f32 q12, q3, q12
        
    
        
        
            vadd.f32 q13, q3, q13
        
    

    vdup.f32 q3, d2[1]
    
        
        
            vadd.f32 q14, q3, q14
        
    
        
        
            vadd.f32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_sub:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    
        vldmia      r3!, { d2 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vsub.f32 q4, q3, q4
        
    
        
        
            vsub.f32 q5, q3, q5
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vsub.f32 q6, q3, q6
        
    
        
        
            vsub.f32 q7, q3, q7
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vsub.f32 q8, q3, q8
        
    
        
        
            vsub.f32 q9, q3, q9
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vsub.f32 q10, q3, q10
        
    
        
        
            vsub.f32 q11, q3, q11
        
    

    vdup.f32 q3, d2[0]
    
        
        
            vsub.f32 q12, q3, q12
        
    
        
        
            vsub.f32 q13, q3, q13
        
    

    vdup.f32 q3, d2[1]
    
        
        
            vsub.f32 q14, q3, q14
        
    
        
        
            vsub.f32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_sub_flipped:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    
        vldmia      r3!, { d2 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vsub.f32 q4, q4, q3
        
    
        
        
            vsub.f32 q5, q5, q3
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vsub.f32 q6, q6, q3
        
    
        
        
            vsub.f32 q7, q7, q3
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vsub.f32 q8, q8, q3
        
    
        
        
            vsub.f32 q9, q9, q3
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vsub.f32 q10, q10, q3
        
    
        
        
            vsub.f32 q11, q11, q3
        
    

    vdup.f32 q3, d2[0]
    
        
        
            vsub.f32 q12, q12, q3
        
    
        
        
            vsub.f32 q13, q13, q3
        
    

    vdup.f32 q3, d2[1]
    
        
        
            vsub.f32 q14, q14, q3
        
    
        
        
            vsub.f32 q15, q15, q3
        
    


    b .non_linear_loop




.add_unicast:
    //  r3, r4, r5, r6 <- ptr, rsc, csc, size
    
        mov         r2, r3
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d8, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d9, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d10, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d11, d0
        
        add r3, r3, r5
    
        mov         r2, r3
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d12, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d13, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d14, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d15, d0
        
        add r3, r3, r5
    
        mov         r2, r3
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d16, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d17, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d18, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d19, d0
        
        add r3, r3, r5
    
        mov         r2, r3
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d20, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d21, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d22, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d23, d0
        
        add r3, r3, r5
    
        mov         r2, r3
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d24, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d25, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d26, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d27, d0
        
        add r3, r3, r5
    
        mov         r2, r3
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d28, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d29, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d30, d0
        
            vld1.f32    d0[0], [ r2 ], r4
            vld1.f32    d0[1], [ r2 ], r4
            vadd.f32    d31, d0
        
        add r3, r3, r5
    

    b .non_linear_loop

.add_row_col_products:
    vldmia          r3!, { q0, q1 }
    vldmia          r4!, { d4, d5, d6 }

    vmla.f32        q4, q0, d4[0]
    vmla.f32        q5, q1, d4[0]

    vmla.f32        q6, q0, d4[1]
    vmla.f32        q7, q1, d4[1]

    vmla.f32        q8, q0, d5[0]
    vmla.f32        q9, q1, d5[0]

    vmla.f32        q10, q0, d5[1]
    vmla.f32        q11, q1, d5[1]

    vmla.f32        q12, q0, d6[0]
    vmla.f32        q13, q1, d6[0]

    vmla.f32        q14, q0, d6[1]
    vmla.f32        q15, q1, d6[1]

    b .non_linear_loop

.store:
    // r3, r4, r5 <- ptr, rsc, csc

    cmp     r4, #4
    bne     .store_generic

    
        mov         r8, r3
        
            vst1.64     d8, [ r8 ]!
        
            vst1.64     d9, [ r8 ]!
        
            vst1.64     d10, [ r8 ]!
        
            vst1.64     d11, [ r8 ]!
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.64     d12, [ r8 ]!
        
            vst1.64     d13, [ r8 ]!
        
            vst1.64     d14, [ r8 ]!
        
            vst1.64     d15, [ r8 ]!
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.64     d16, [ r8 ]!
        
            vst1.64     d17, [ r8 ]!
        
            vst1.64     d18, [ r8 ]!
        
            vst1.64     d19, [ r8 ]!
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.64     d20, [ r8 ]!
        
            vst1.64     d21, [ r8 ]!
        
            vst1.64     d22, [ r8 ]!
        
            vst1.64     d23, [ r8 ]!
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.64     d24, [ r8 ]!
        
            vst1.64     d25, [ r8 ]!
        
            vst1.64     d26, [ r8 ]!
        
            vst1.64     d27, [ r8 ]!
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.64     d28, [ r8 ]!
        
            vst1.64     d29, [ r8 ]!
        
            vst1.64     d30, [ r8 ]!
        
            vst1.64     d31, [ r8 ]!
        
        
    

    b .non_linear_loop

.store_generic:
    
        mov         r8, r3
        
            vst1.f32    d8[0], [ r8 ], r4
            vst1.f32    d8[1], [ r8 ], r4
        
            vst1.f32    d9[0], [ r8 ], r4
            vst1.f32    d9[1], [ r8 ], r4
        
            vst1.f32    d10[0], [ r8 ], r4
            vst1.f32    d10[1], [ r8 ], r4
        
            vst1.f32    d11[0], [ r8 ], r4
            vst1.f32    d11[1], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.f32    d12[0], [ r8 ], r4
            vst1.f32    d12[1], [ r8 ], r4
        
            vst1.f32    d13[0], [ r8 ], r4
            vst1.f32    d13[1], [ r8 ], r4
        
            vst1.f32    d14[0], [ r8 ], r4
            vst1.f32    d14[1], [ r8 ], r4
        
            vst1.f32    d15[0], [ r8 ], r4
            vst1.f32    d15[1], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.f32    d16[0], [ r8 ], r4
            vst1.f32    d16[1], [ r8 ], r4
        
            vst1.f32    d17[0], [ r8 ], r4
            vst1.f32    d17[1], [ r8 ], r4
        
            vst1.f32    d18[0], [ r8 ], r4
            vst1.f32    d18[1], [ r8 ], r4
        
            vst1.f32    d19[0], [ r8 ], r4
            vst1.f32    d19[1], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.f32    d20[0], [ r8 ], r4
            vst1.f32    d20[1], [ r8 ], r4
        
            vst1.f32    d21[0], [ r8 ], r4
            vst1.f32    d21[1], [ r8 ], r4
        
            vst1.f32    d22[0], [ r8 ], r4
            vst1.f32    d22[1], [ r8 ], r4
        
            vst1.f32    d23[0], [ r8 ], r4
            vst1.f32    d23[1], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.f32    d24[0], [ r8 ], r4
            vst1.f32    d24[1], [ r8 ], r4
        
            vst1.f32    d25[0], [ r8 ], r4
            vst1.f32    d25[1], [ r8 ], r4
        
            vst1.f32    d26[0], [ r8 ], r4
            vst1.f32    d26[1], [ r8 ], r4
        
            vst1.f32    d27[0], [ r8 ], r4
            vst1.f32    d27[1], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vst1.f32    d28[0], [ r8 ], r4
            vst1.f32    d28[1], [ r8 ], r4
        
            vst1.f32    d29[0], [ r8 ], r4
            vst1.f32    d29[1], [ r8 ], r4
        
            vst1.f32    d30[0], [ r8 ], r4
            vst1.f32    d30[1], [ r8 ], r4
        
            vst1.f32    d31[0], [ r8 ], r4
            vst1.f32    d31[1], [ r8 ], r4
        
        
    

    b .non_linear_loop

.load_tile:
    vldmia          r3!, { q4-q7 }
    vldmia          r3!, { q8-q15 }
    b .non_linear_loop

.return:
    vpop        { q4-q7 }
    pop         { r4-r12 }

    bx          lr


// vim: ft=arm

// C tile regs
// 
//      q8[0]    q10[0]   q12[0]    q14[0]
//      q8[1]    q10[1]   q12[1]    q14[1]
//      q8[2]    q10[2]   q12[2]    q14[2]
//      q8[3]    q10[3]   q12[3]    q14[3]
//
//      q9[0]    q11[0]   q13[0]    q15[0]
//      q9[1]    q11[1]   q13[1]    q15[1]
//      q9[2]    q11[2]   q13[2]    q15[2]
//      q9[3]    q11[3]   q13[3]    q15[3]

    .arm
    .text
    .global armv7neon_mmm_i32_8x4_0_21_10
    .type armv7neon_mmm_i32_8x4_0_21_10, %function

armv7neon_mmm_i32_8x4_0_21_10:

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
    // r3 r4 r5 r6
    // k  a  b  packing
    cmp     r3, #0
    beq     .non_linear_loop

    mov     r1, r4 // packed A ptr
    pld     [r3]
    pld     [r5]

    cmp     r6, #1
    beq     .packed_packed_i8i8

    .packed_packed_loop_1:

    vldmia          r1!, { q0, q1 }
    vldmia          r5!, { q2 }

    vmla.s32       q8, q0, d4[0]
    vmla.s32       q9, q1, d4[0]

    vmla.s32       q10, q0, d4[1]
    vmla.s32       q11, q1, d4[1]

    vmla.s32       q12, q0, d5[0]
    vmla.s32       q13, q1, d5[0]

    vmla.s32       q14, q0, d5[1]
    vmla.s32       q15, q1, d5[1]

    subs r3, r3, #1
    bne .packed_packed_loop_1

    b   .non_linear_loop

    .packed_packed_i8i8:
    pld     [r5]                           // packed B ptr       

    cmp r3, #4
    blt .packed_packed_loop_i8i8_1

    .packed_packed_loop_i8i8_4:
    pld             [r1, #64]
    pld             [r5, #64]

    // q2: d4 -> d4,d5 A even cols (from r1)
    // q3: d6 -> d6,d7 A odd cols (from r1)
    // q0: s0 -> d0 : B even lines (from r5)
    // q1: s4 -> d2 : B odd lines (from r5)

    // 0
    vldmia          r1!, { d4 }
    vldmia          r5!, { s0 }

    vmovl.s8        q2, d4
    vmovl.s8        q0, d0

    vmlal.s16       q8, d4, d0[0]
    vmlal.s16       q9, d5, d0[0]

    vldmia          r1!, { d6 }

    vmlal.s16       q10, d4, d0[1]
    vmlal.s16       q11, d5, d0[1]

    vldmia          r5!, { s4 }

    vmlal.s16       q12, d4, d0[2]
    vmlal.s16       q13, d5, d0[2]

    vmlal.s16       q14, d4, d0[3]
    vmlal.s16       q15, d5, d0[3]

    // 1
    vmovl.s8        q3, d6
    vmovl.s8        q1, d2

    vmlal.s16       q8, d6, d2[0]
    vldmia          r1!, { d4 }
    vmlal.s16       q9, d7, d2[0]
    vldmia          r5!, { s0 }

    vmlal.s16       q10, d6, d2[1]
    vmlal.s16       q11, d7, d2[1]

    vmlal.s16       q12, d6, d2[2]
    vmlal.s16       q13, d7, d2[2]

    vmlal.s16       q14, d6, d2[3]
    vmlal.s16       q15, d7, d2[3]

    // 2
    vmovl.s8        q2, d4
    vmovl.s8        q0, d0

    vmlal.s16       q8, d4, d0[0]
    vmlal.s16       q9, d5, d0[0]

    vldmia          r1!, { d6 }

    vmlal.s16       q10, d4, d0[1]
    vmlal.s16       q11, d5, d0[1]

    vldmia          r5!, { s4 }

    vmlal.s16       q12, d4, d0[2]
    vmlal.s16       q13, d5, d0[2]

    vmlal.s16       q14, d4, d0[3]
    vmlal.s16       q15, d5, d0[3]

    // 3
    vmovl.s8        q3, d6
    vmovl.s8        q1, d2

    vmlal.s16       q8, d6, d2[0]
    vmlal.s16       q9, d7, d2[0]

    vmlal.s16       q10, d6, d2[1]
    vmlal.s16       q11, d7, d2[1]

    vmlal.s16       q12, d6, d2[2]
    vmlal.s16       q13, d7, d2[2]

    vmlal.s16       q14, d6, d2[3]
    vmlal.s16       q15, d7, d2[3]

    sub r3, r3, #4
    cmp r3, #4
    bge .packed_packed_loop_i8i8_4

    cmp r3, #0
    beq .non_linear_loop

    .packed_packed_loop_i8i8_1:

    vldmia          r1!, { s0, s1 }
    vmovl.s8        q0, d0
    vldmia          r5!, { s4 }
    vmovl.s8        q1, d2

    vmlal.s16       q8, d0, d2[0]
    vmlal.s16       q9, d1, d2[0]

    vmlal.s16       q10, d0, d2[1]
    vmlal.s16       q11, d1, d2[1]

    vmlal.s16       q12, d0, d2[2]
    vmlal.s16       q13, d1, d2[2]

    vmlal.s16       q14, d0, d2[3]
    vmlal.s16       q15, d1, d2[3]

    subs r3, r3, #1
    bne .packed_packed_loop_i8i8_1
    b   .non_linear_loop

// vim: ft=arm

// vim: ft=arm

.scalar_min:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmin.s32    q8, q0, q8
        
            vmin.s32    q9, q0, q9
        
            vmin.s32    q10, q0, q10
        
            vmin.s32    q11, q0, q11
        
            vmin.s32    q12, q0, q12
        
            vmin.s32    q13, q0, q13
        
            vmin.s32    q14, q0, q14
        
            vmin.s32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_max:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmax.s32    q8, q0, q8
        
            vmax.s32    q9, q0, q9
        
            vmax.s32    q10, q0, q10
        
            vmax.s32    q11, q0, q11
        
            vmax.s32    q12, q0, q12
        
            vmax.s32    q13, q0, q13
        
            vmax.s32    q14, q0, q14
        
            vmax.s32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_mul:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vmul.s32    q8, q0, q8
        
            vmul.s32    q9, q0, q9
        
            vmul.s32    q10, q0, q10
        
            vmul.s32    q11, q0, q11
        
            vmul.s32    q12, q0, q12
        
            vmul.s32    q13, q0, q13
        
            vmul.s32    q14, q0, q14
        
            vmul.s32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_add:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vadd.s32    q8, q0, q8
        
            vadd.s32    q9, q0, q9
        
            vadd.s32    q10, q0, q10
        
            vadd.s32    q11, q0, q11
        
            vadd.s32    q12, q0, q12
        
            vadd.s32    q13, q0, q13
        
            vadd.s32    q14, q0, q14
        
            vadd.s32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_sub:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vsub.s32    q8, q0, q8
        
            vsub.s32    q9, q0, q9
        
            vsub.s32    q10, q0, q10
        
            vsub.s32    q11, q0, q11
        
            vsub.s32    q12, q0, q12
        
            vsub.s32    q13, q0, q13
        
            vsub.s32    q14, q0, q14
        
            vsub.s32    q15, q0, q15
        
    
    b .non_linear_loop

// vim: ft=arm

.scalar_sub_flipped:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        
            vsub.s32    q8, q8, q0
        
            vsub.s32    q9, q9, q0
        
            vsub.s32    q10, q10, q0
        
            vsub.s32    q11, q11, q0
        
            vsub.s32    q12, q12, q0
        
            vsub.s32    q13, q13, q0
        
            vsub.s32    q14, q14, q0
        
            vsub.s32    q15, q15, q0
        
    
    b .non_linear_loop


.leaky_relu:
    vmov            s0, r3
    vdup.32         q0, d0[0]
    
        vmul.s32    q2, q8, q0
        vcgt.s32    q1, q8, 0
        vbsl        q1, q8, q2
        vmov        q8, q1
    
        vmul.s32    q2, q9, q0
        vcgt.s32    q1, q9, 0
        vbsl        q1, q9, q2
        vmov        q9, q1
    
        vmul.s32    q2, q10, q0
        vcgt.s32    q1, q10, 0
        vbsl        q1, q10, q2
        vmov        q10, q1
    
        vmul.s32    q2, q11, q0
        vcgt.s32    q1, q11, 0
        vbsl        q1, q11, q2
        vmov        q11, q1
    
        vmul.s32    q2, q12, q0
        vcgt.s32    q1, q12, 0
        vbsl        q1, q12, q2
        vmov        q12, q1
    
        vmul.s32    q2, q13, q0
        vcgt.s32    q1, q13, 0
        vbsl        q1, q13, q2
        vmov        q13, q1
    
        vmul.s32    q2, q14, q0
        vcgt.s32    q1, q14, 0
        vbsl        q1, q14, q2
        vmov        q14, q1
    
        vmul.s32    q2, q15, q0
        vcgt.s32    q1, q15, 0
        vbsl        q1, q15, q2
        vmov        q15, q1
    
    b .non_linear_loop


// vim: ft=arm

// vim: ft=arm

.per_row_min:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmin.s32 q8, q0, q8
    
        
        vmin.s32 q9, q1, q9
    
        
        vmin.s32 q10, q0, q10
    
        
        vmin.s32 q11, q1, q11
    
        
        vmin.s32 q12, q0, q12
    
        
        vmin.s32 q13, q1, q13
    
        
        vmin.s32 q14, q0, q14
    
        
        vmin.s32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_max:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmax.s32 q8, q0, q8
    
        
        vmax.s32 q9, q1, q9
    
        
        vmax.s32 q10, q0, q10
    
        
        vmax.s32 q11, q1, q11
    
        
        vmax.s32 q12, q0, q12
    
        
        vmax.s32 q13, q1, q13
    
        
        vmax.s32 q14, q0, q14
    
        
        vmax.s32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_mul:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vmul.s32 q8, q0, q8
    
        
        vmul.s32 q9, q1, q9
    
        
        vmul.s32 q10, q0, q10
    
        
        vmul.s32 q11, q1, q11
    
        
        vmul.s32 q12, q0, q12
    
        
        vmul.s32 q13, q1, q13
    
        
        vmul.s32 q14, q0, q14
    
        
        vmul.s32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_add:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vadd.s32 q8, q0, q8
    
        
        vadd.s32 q9, q1, q9
    
        
        vadd.s32 q10, q0, q10
    
        
        vadd.s32 q11, q1, q11
    
        
        vadd.s32 q12, q0, q12
    
        
        vadd.s32 q13, q1, q13
    
        
        vadd.s32 q14, q0, q14
    
        
        vadd.s32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_sub:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vsub.s32 q8, q0, q8
    
        
        vsub.s32 q9, q1, q9
    
        
        vsub.s32 q10, q0, q10
    
        
        vsub.s32 q11, q1, q11
    
        
        vsub.s32 q12, q0, q12
    
        
        vsub.s32 q13, q1, q13
    
        
        vsub.s32 q14, q0, q14
    
        
        vsub.s32 q15, q1, q15
    


b           .non_linear_loop

// vim: ft=arm

.per_row_sub_flipped:





    vldmia         r3!, { q0 }

    vldmia         r3!, { q1 }



    
        
        vsub.s32 q8, q8, q0
    
        
        vsub.s32 q9, q9, q1
    
        
        vsub.s32 q10, q10, q0
    
        
        vsub.s32 q11, q11, q1
    
        
        vsub.s32 q12, q12, q0
    
        
        vsub.s32 q13, q13, q1
    
        
        vsub.s32 q14, q14, q0
    
        
        vsub.s32 q15, q15, q1
    


b           .non_linear_loop


// vim: ft=arm

// vim: ft=arm

.per_col_min:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vmin.s32 q8, q3, q8
        
    
        
        
            vmin.s32 q9, q3, q9
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vmin.s32 q10, q3, q10
        
    
        
        
            vmin.s32 q11, q3, q11
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vmin.s32 q12, q3, q12
        
    
        
        
            vmin.s32 q13, q3, q13
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vmin.s32 q14, q3, q14
        
    
        
        
            vmin.s32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_max:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vmax.s32 q8, q3, q8
        
    
        
        
            vmax.s32 q9, q3, q9
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vmax.s32 q10, q3, q10
        
    
        
        
            vmax.s32 q11, q3, q11
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vmax.s32 q12, q3, q12
        
    
        
        
            vmax.s32 q13, q3, q13
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vmax.s32 q14, q3, q14
        
    
        
        
            vmax.s32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_mul:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vmul.s32 q8, q3, q8
        
    
        
        
            vmul.s32 q9, q3, q9
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vmul.s32 q10, q3, q10
        
    
        
        
            vmul.s32 q11, q3, q11
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vmul.s32 q12, q3, q12
        
    
        
        
            vmul.s32 q13, q3, q13
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vmul.s32 q14, q3, q14
        
    
        
        
            vmul.s32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_add:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vadd.s32 q8, q3, q8
        
    
        
        
            vadd.s32 q9, q3, q9
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vadd.s32 q10, q3, q10
        
    
        
        
            vadd.s32 q11, q3, q11
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vadd.s32 q12, q3, q12
        
    
        
        
            vadd.s32 q13, q3, q13
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vadd.s32 q14, q3, q14
        
    
        
        
            vadd.s32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_sub:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vsub.s32 q8, q3, q8
        
    
        
        
            vsub.s32 q9, q3, q9
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vsub.s32 q10, q3, q10
        
    
        
        
            vsub.s32 q11, q3, q11
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vsub.s32 q12, q3, q12
        
    
        
        
            vsub.s32 q13, q3, q13
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vsub.s32 q14, q3, q14
        
    
        
        
            vsub.s32 q15, q3, q15
        
    


    b .non_linear_loop

// vim: ft=arm

.per_col_sub_flipped:








    
    
        vldmia      r3!, { d0 }
    
        vldmia      r3!, { d1 }
    




    vdup.f32 q3, d0[0]
    
        
        
            vsub.s32 q8, q8, q3
        
    
        
        
            vsub.s32 q9, q9, q3
        
    

    vdup.f32 q3, d0[1]
    
        
        
            vsub.s32 q10, q10, q3
        
    
        
        
            vsub.s32 q11, q11, q3
        
    

    vdup.f32 q3, d1[0]
    
        
        
            vsub.s32 q12, q12, q3
        
    
        
        
            vsub.s32 q13, q13, q3
        
    

    vdup.f32 q3, d1[1]
    
        
        
            vsub.s32 q14, q14, q3
        
    
        
        
            vsub.s32 q15, q15, q3
        
    


    b .non_linear_loop



.add_unicast:
    // r3, r4, r5, r6 <- ptr, rsc, csc, size
    cmp     r6, #4
    beq     .non_linear_addc_i32

    
        mov         r8, r3
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d16, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d17, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d18, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d19, d0
        
        add r3, r3, r5
    
        mov         r8, r3
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d20, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d21, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d22, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d23, d0
        
        add r3, r3, r5
    
        mov         r8, r3
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d24, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d25, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d26, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d27, d0
        
        add r3, r3, r5
    
        mov         r8, r3
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d28, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d29, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d30, d0
        
            vld1.s8     d0[0], [ r8 ], r4
            vld1.s8     d0[1], [ r8 ], r4
            vmovl.s8    q0, d0
            vmovl.s16   q0, d0
            vadd.i32    d31, d0
        
        add r3, r3, r5
    

    b .non_linear_loop

.non_linear_addc_i32:

    
        mov         r8, r3
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d16, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d17, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d18, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d19, d0
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d20, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d21, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d22, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d23, d0
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d24, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d25, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d26, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d27, d0
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d28, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d29, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d30, d0
        
            vld1.s32    d0[0], [ r8 ], r4
            vld1.s32    d0[1], [ r8 ], r4
            vadd.i32    d31, d0
        
        
    

b .non_linear_loop

.add_row_col_products:
    vldmia          r3!, { q0, q1 }
    vldmia          r4!, { q4 }

    vmla.s32        q8, q0, d8[0]
    vmla.s32        q9, q1, d8[0]

    vmla.s32        q10, q0, d8[1]
    vmla.s32        q11, q1, d8[1]

    vmla.s32        q12, q0, d9[0]
    vmla.s32        q13, q1, d9[0]

    vmla.s32        q14, q0, d9[1]
    vmla.s32        q15, q1, d9[1]

    b .non_linear_loop

    // vim: ft=arm

.q_scale:
    ldm         r0, { r4, r5, r6, r7 }      // fixme params are already loaded by disp.
    vdup.s32    q0, r7                      // q0 <- multiplier

    mov         r3, #1
    vdup.s32    q1, r3                      // q1 <- ones
    vmovl.s32   q1, d2

    add         r5, #32
    neg         r5, r5
    vdup.s32    q2, r5                      // q2 <- -(shift + 32)
    vmovl.s32   q2, d4

    cmp     r6, #1
    beq     .q_scale_rounding_zero
    cmp     r6, #2
    beq     .q_scale_rounding_away
    cmp     r6, #3
    beq     .q_scale_rounding_minus_inf
    cmp     r6, #4
    beq     .q_scale_rounding_plus_inf
    cmp     r6, #5
    beq     .q_scale_rounding_even
    cmp     r6, #6
    beq     .q_scale_rounding_odd

    b .unsupported

.q_scale_rounding_zero:
    
        vclt.s32    q7, q8, #0
        vabs.s32    q8, q8
        vqdmull.s32 q5, d16, d0[0]
        vqdmull.s32 q6, d17, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d16, q5
        vmovn.s64   d17, q6
        vneg.s32    q5, q8
        vbit.s32    q8, q5, q7
    
        vclt.s32    q7, q9, #0
        vabs.s32    q9, q9
        vqdmull.s32 q5, d18, d0[0]
        vqdmull.s32 q6, d19, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d18, q5
        vmovn.s64   d19, q6
        vneg.s32    q5, q9
        vbit.s32    q9, q5, q7
    
        vclt.s32    q7, q10, #0
        vabs.s32    q10, q10
        vqdmull.s32 q5, d20, d0[0]
        vqdmull.s32 q6, d21, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d20, q5
        vmovn.s64   d21, q6
        vneg.s32    q5, q10
        vbit.s32    q10, q5, q7
    
        vclt.s32    q7, q11, #0
        vabs.s32    q11, q11
        vqdmull.s32 q5, d22, d0[0]
        vqdmull.s32 q6, d23, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d22, q5
        vmovn.s64   d23, q6
        vneg.s32    q5, q11
        vbit.s32    q11, q5, q7
    
        vclt.s32    q7, q12, #0
        vabs.s32    q12, q12
        vqdmull.s32 q5, d24, d0[0]
        vqdmull.s32 q6, d25, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d24, q5
        vmovn.s64   d25, q6
        vneg.s32    q5, q12
        vbit.s32    q12, q5, q7
    
        vclt.s32    q7, q13, #0
        vabs.s32    q13, q13
        vqdmull.s32 q5, d26, d0[0]
        vqdmull.s32 q6, d27, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d26, q5
        vmovn.s64   d27, q6
        vneg.s32    q5, q13
        vbit.s32    q13, q5, q7
    
        vclt.s32    q7, q14, #0
        vabs.s32    q14, q14
        vqdmull.s32 q5, d28, d0[0]
        vqdmull.s32 q6, d29, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d28, q5
        vmovn.s64   d29, q6
        vneg.s32    q5, q14
        vbit.s32    q14, q5, q7
    
        vclt.s32    q7, q15, #0
        vabs.s32    q15, q15
        vqdmull.s32 q5, d30, d0[0]
        vqdmull.s32 q6, d31, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d30, q5
        vmovn.s64   d31, q6
        vneg.s32    q5, q15
        vbit.s32    q15, q5, q7
    

    b .non_linear_loop

.q_scale_rounding_away:
    
        vclt.s32    q7, q8, #0
        vabs.s32    q8, q8
        vqdmull.s32 q5, d16, d0[0]
        vqdmull.s32 q6, d17, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d16, q5
        vmovn.s64   d17, q6
        vneg.s32    q5, q8
        vbit.s32    q8, q5, q7
    
        vclt.s32    q7, q9, #0
        vabs.s32    q9, q9
        vqdmull.s32 q5, d18, d0[0]
        vqdmull.s32 q6, d19, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d18, q5
        vmovn.s64   d19, q6
        vneg.s32    q5, q9
        vbit.s32    q9, q5, q7
    
        vclt.s32    q7, q10, #0
        vabs.s32    q10, q10
        vqdmull.s32 q5, d20, d0[0]
        vqdmull.s32 q6, d21, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d20, q5
        vmovn.s64   d21, q6
        vneg.s32    q5, q10
        vbit.s32    q10, q5, q7
    
        vclt.s32    q7, q11, #0
        vabs.s32    q11, q11
        vqdmull.s32 q5, d22, d0[0]
        vqdmull.s32 q6, d23, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d22, q5
        vmovn.s64   d23, q6
        vneg.s32    q5, q11
        vbit.s32    q11, q5, q7
    
        vclt.s32    q7, q12, #0
        vabs.s32    q12, q12
        vqdmull.s32 q5, d24, d0[0]
        vqdmull.s32 q6, d25, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d24, q5
        vmovn.s64   d25, q6
        vneg.s32    q5, q12
        vbit.s32    q12, q5, q7
    
        vclt.s32    q7, q13, #0
        vabs.s32    q13, q13
        vqdmull.s32 q5, d26, d0[0]
        vqdmull.s32 q6, d27, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d26, q5
        vmovn.s64   d27, q6
        vneg.s32    q5, q13
        vbit.s32    q13, q5, q7
    
        vclt.s32    q7, q14, #0
        vabs.s32    q14, q14
        vqdmull.s32 q5, d28, d0[0]
        vqdmull.s32 q6, d29, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d28, q5
        vmovn.s64   d29, q6
        vneg.s32    q5, q14
        vbit.s32    q14, q5, q7
    
        vclt.s32    q7, q15, #0
        vabs.s32    q15, q15
        vqdmull.s32 q5, d30, d0[0]
        vqdmull.s32 q6, d31, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d30, q5
        vmovn.s64   d31, q6
        vneg.s32    q5, q15
        vbit.s32    q15, q5, q7
    

    b .non_linear_loop

.q_scale_rounding_minus_inf:
    
        vqdmull.s32 q5, d16, d0[0]
        vqdmull.s32 q6, d17, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d16, q5
        vmovn.s64   d17, q6
    
        vqdmull.s32 q5, d18, d0[0]
        vqdmull.s32 q6, d19, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d18, q5
        vmovn.s64   d19, q6
    
        vqdmull.s32 q5, d20, d0[0]
        vqdmull.s32 q6, d21, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d20, q5
        vmovn.s64   d21, q6
    
        vqdmull.s32 q5, d22, d0[0]
        vqdmull.s32 q6, d23, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d22, q5
        vmovn.s64   d23, q6
    
        vqdmull.s32 q5, d24, d0[0]
        vqdmull.s32 q6, d25, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d24, q5
        vmovn.s64   d25, q6
    
        vqdmull.s32 q5, d26, d0[0]
        vqdmull.s32 q6, d27, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d26, q5
        vmovn.s64   d27, q6
    
        vqdmull.s32 q5, d28, d0[0]
        vqdmull.s32 q6, d29, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d28, q5
        vmovn.s64   d29, q6
    
        vqdmull.s32 q5, d30, d0[0]
        vqdmull.s32 q6, d31, d0[0]
        vsub.s64    q5, q1
        vsub.s64    q6, q1
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d30, q5
        vmovn.s64   d31, q6
    

    b .non_linear_loop

.q_scale_rounding_plus_inf:
    
        vqdmull.s32 q5, d16, d0[0]
        vqdmull.s32 q6, d17, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d16, q5
        vmovn.s64   d17, q6
    
        vqdmull.s32 q5, d18, d0[0]
        vqdmull.s32 q6, d19, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d18, q5
        vmovn.s64   d19, q6
    
        vqdmull.s32 q5, d20, d0[0]
        vqdmull.s32 q6, d21, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d20, q5
        vmovn.s64   d21, q6
    
        vqdmull.s32 q5, d22, d0[0]
        vqdmull.s32 q6, d23, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d22, q5
        vmovn.s64   d23, q6
    
        vqdmull.s32 q5, d24, d0[0]
        vqdmull.s32 q6, d25, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d24, q5
        vmovn.s64   d25, q6
    
        vqdmull.s32 q5, d26, d0[0]
        vqdmull.s32 q6, d27, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d26, q5
        vmovn.s64   d27, q6
    
        vqdmull.s32 q5, d28, d0[0]
        vqdmull.s32 q6, d29, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d28, q5
        vmovn.s64   d29, q6
    
        vqdmull.s32 q5, d30, d0[0]
        vqdmull.s32 q6, d31, d0[0]
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d30, q5
        vmovn.s64   d31, q6
    

    b .non_linear_loop

.q_scale_rounding_even:
    
        vclt.s32    q7, q8, #0
        vabs.s32    q8, q8
        vqdmull.s32 q5, d16, d0[0]
        vqdmull.s32 q6, d17, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d16, q5
        vmovn.s64   d17, q6
        vneg.s32    q5, q8
        vbit.s32    q8, q5, q7
    
        vclt.s32    q7, q9, #0
        vabs.s32    q9, q9
        vqdmull.s32 q5, d18, d0[0]
        vqdmull.s32 q6, d19, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d18, q5
        vmovn.s64   d19, q6
        vneg.s32    q5, q9
        vbit.s32    q9, q5, q7
    
        vclt.s32    q7, q10, #0
        vabs.s32    q10, q10
        vqdmull.s32 q5, d20, d0[0]
        vqdmull.s32 q6, d21, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d20, q5
        vmovn.s64   d21, q6
        vneg.s32    q5, q10
        vbit.s32    q10, q5, q7
    
        vclt.s32    q7, q11, #0
        vabs.s32    q11, q11
        vqdmull.s32 q5, d22, d0[0]
        vqdmull.s32 q6, d23, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d22, q5
        vmovn.s64   d23, q6
        vneg.s32    q5, q11
        vbit.s32    q11, q5, q7
    
        vclt.s32    q7, q12, #0
        vabs.s32    q12, q12
        vqdmull.s32 q5, d24, d0[0]
        vqdmull.s32 q6, d25, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d24, q5
        vmovn.s64   d25, q6
        vneg.s32    q5, q12
        vbit.s32    q12, q5, q7
    
        vclt.s32    q7, q13, #0
        vabs.s32    q13, q13
        vqdmull.s32 q5, d26, d0[0]
        vqdmull.s32 q6, d27, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d26, q5
        vmovn.s64   d27, q6
        vneg.s32    q5, q13
        vbit.s32    q13, q5, q7
    
        vclt.s32    q7, q14, #0
        vabs.s32    q14, q14
        vqdmull.s32 q5, d28, d0[0]
        vqdmull.s32 q6, d29, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d28, q5
        vmovn.s64   d29, q6
        vneg.s32    q5, q14
        vbit.s32    q14, q5, q7
    
        vclt.s32    q7, q15, #0
        vabs.s32    q15, q15
        vqdmull.s32 q5, d30, d0[0]
        vqdmull.s32 q6, d31, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q3, q3, q1
        vsub.s64    q4, q4, q1
        vadd.s64    q5, q3
        vadd.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d30, q5
        vmovn.s64   d31, q6
        vneg.s32    q5, q15
        vbit.s32    q15, q5, q7
    

    b .non_linear_loop

.q_scale_rounding_odd:
    
        vclt.s32    q7, q8, #0
        vabs.s32    q8, q8
        vqdmull.s32 q5, d16, d0[0]
        vqdmull.s32 q6, d17, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d16, q5
        vmovn.s64   d17, q6
        vneg.s32    q5, q8
        vbit.s32    q8, q5, q7
    
        vclt.s32    q7, q9, #0
        vabs.s32    q9, q9
        vqdmull.s32 q5, d18, d0[0]
        vqdmull.s32 q6, d19, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d18, q5
        vmovn.s64   d19, q6
        vneg.s32    q5, q9
        vbit.s32    q9, q5, q7
    
        vclt.s32    q7, q10, #0
        vabs.s32    q10, q10
        vqdmull.s32 q5, d20, d0[0]
        vqdmull.s32 q6, d21, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d20, q5
        vmovn.s64   d21, q6
        vneg.s32    q5, q10
        vbit.s32    q10, q5, q7
    
        vclt.s32    q7, q11, #0
        vabs.s32    q11, q11
        vqdmull.s32 q5, d22, d0[0]
        vqdmull.s32 q6, d23, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d22, q5
        vmovn.s64   d23, q6
        vneg.s32    q5, q11
        vbit.s32    q11, q5, q7
    
        vclt.s32    q7, q12, #0
        vabs.s32    q12, q12
        vqdmull.s32 q5, d24, d0[0]
        vqdmull.s32 q6, d25, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d24, q5
        vmovn.s64   d25, q6
        vneg.s32    q5, q12
        vbit.s32    q12, q5, q7
    
        vclt.s32    q7, q13, #0
        vabs.s32    q13, q13
        vqdmull.s32 q5, d26, d0[0]
        vqdmull.s32 q6, d27, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d26, q5
        vmovn.s64   d27, q6
        vneg.s32    q5, q13
        vbit.s32    q13, q5, q7
    
        vclt.s32    q7, q14, #0
        vabs.s32    q14, q14
        vqdmull.s32 q5, d28, d0[0]
        vqdmull.s32 q6, d29, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d28, q5
        vmovn.s64   d29, q6
        vneg.s32    q5, q14
        vbit.s32    q14, q5, q7
    
        vclt.s32    q7, q15, #0
        vabs.s32    q15, q15
        vqdmull.s32 q5, d30, d0[0]
        vqdmull.s32 q6, d31, d0[0]
        vqshl.s64   q3, q5, q2
        vqshl.s64   q4, q6, q2
        vand        q3, q3, q1
        vand        q4, q4, q1
        vsub.s64    q5, q3
        vsub.s64    q6, q4
        vqrshl.s64  q5, q2
        vqrshl.s64  q6, q2
        vmovn.s64   d30, q5
        vmovn.s64   d31, q6
        vneg.s32    q5, q15
        vbit.s32    q15, q5, q7
    

    b .non_linear_loop

.q_shl:
    ldm         r0, { r4, r5 }      // fixme params are already loaded by disp.
    vdup.s32    q2, r5              // q2 <- shift

    
        vqrshl.s32  q8, q2      // Shift
    
        vqrshl.s32  q9, q2      // Shift
    
        vqrshl.s32  q10, q2      // Shift
    
        vqrshl.s32  q11, q2      // Shift
    
        vqrshl.s32  q12, q2      // Shift
    
        vqrshl.s32  q13, q2      // Shift
    
        vqrshl.s32  q14, q2      // Shift
    
        vqrshl.s32  q15, q2      // Shift
    

    b .non_linear_loop

.q_shr:
    ldm         r0, { r4, r5, r6 }      // fixme params are already loaded by disp.

    mov         r3, #1
    vdup.s32    q1, r3                      // q1 <- ones

    neg         r5, r5
    vdup.s32    q2, r5                      // q2 <- shift

    cmp     r6, #1
    beq     .q_shr_rounding_zero
    cmp     r6, #2
    beq     .q_shr_rounding_away
    cmp     r6, #3
    beq     .q_shr_rounding_minus_inf
    cmp     r6, #4
    beq     .q_shr_rounding_plus_inf
    cmp     r6, #5
    beq     .q_shr_rounding_even
    cmp     r6, #6
    beq     .q_shr_rounding_odd

    b .unsupported

.q_shr_rounding_zero:
    // return signum(x) * ((abs(x) - 1) >>r shift )
    
        vclt.s32    q3, q8, #0  // Store the sign of the value
        vabs.s32    q8, q8  // Compute their abs
        vsub.s32    q8, q1      // Substract 1 to abs(x)
        vqrshl.s32  q8, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q8      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q8, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q9, #0  // Store the sign of the value
        vabs.s32    q9, q9  // Compute their abs
        vsub.s32    q9, q1      // Substract 1 to abs(x)
        vqrshl.s32  q9, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q9      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q9, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q10, #0  // Store the sign of the value
        vabs.s32    q10, q10  // Compute their abs
        vsub.s32    q10, q1      // Substract 1 to abs(x)
        vqrshl.s32  q10, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q10      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q10, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q11, #0  // Store the sign of the value
        vabs.s32    q11, q11  // Compute their abs
        vsub.s32    q11, q1      // Substract 1 to abs(x)
        vqrshl.s32  q11, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q11      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q11, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q12, #0  // Store the sign of the value
        vabs.s32    q12, q12  // Compute their abs
        vsub.s32    q12, q1      // Substract 1 to abs(x)
        vqrshl.s32  q12, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q12      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q12, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q13, #0  // Store the sign of the value
        vabs.s32    q13, q13  // Compute their abs
        vsub.s32    q13, q1      // Substract 1 to abs(x)
        vqrshl.s32  q13, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q13      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q13, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q14, #0  // Store the sign of the value
        vabs.s32    q14, q14  // Compute their abs
        vsub.s32    q14, q1      // Substract 1 to abs(x)
        vqrshl.s32  q14, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q14      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q14, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q15, #0  // Store the sign of the value
        vabs.s32    q15, q15  // Compute their abs
        vsub.s32    q15, q1      // Substract 1 to abs(x)
        vqrshl.s32  q15, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q15      // Compute -((abs(x) - 1) >>r shift )
        vbit.s32    q15, q4, q3  // Restore sign of x with bit mask
    
    b .non_linear_loop

.q_shr_rounding_away:
    // return signum(x) * (abs(x) >>r shift )
    
        vclt.s32    q3, q8, #0  // Store the sign of the value
        vabs.s32    q8, q8  // Compute their abs
        vqrshl.s32  q8, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q8      // Compute -(abs(x) >>r shift )
        vbit.s32    q8, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q9, #0  // Store the sign of the value
        vabs.s32    q9, q9  // Compute their abs
        vqrshl.s32  q9, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q9      // Compute -(abs(x) >>r shift )
        vbit.s32    q9, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q10, #0  // Store the sign of the value
        vabs.s32    q10, q10  // Compute their abs
        vqrshl.s32  q10, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q10      // Compute -(abs(x) >>r shift )
        vbit.s32    q10, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q11, #0  // Store the sign of the value
        vabs.s32    q11, q11  // Compute their abs
        vqrshl.s32  q11, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q11      // Compute -(abs(x) >>r shift )
        vbit.s32    q11, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q12, #0  // Store the sign of the value
        vabs.s32    q12, q12  // Compute their abs
        vqrshl.s32  q12, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q12      // Compute -(abs(x) >>r shift )
        vbit.s32    q12, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q13, #0  // Store the sign of the value
        vabs.s32    q13, q13  // Compute their abs
        vqrshl.s32  q13, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q13      // Compute -(abs(x) >>r shift )
        vbit.s32    q13, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q14, #0  // Store the sign of the value
        vabs.s32    q14, q14  // Compute their abs
        vqrshl.s32  q14, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q14      // Compute -(abs(x) >>r shift )
        vbit.s32    q14, q4, q3  // Restore sign of x with bit mask
    
        vclt.s32    q3, q15, #0  // Store the sign of the value
        vabs.s32    q15, q15  // Compute their abs
        vqrshl.s32  q15, q2      // Rounding shift (0.5 -> 1)
        vneg.s32    q4, q15      // Compute -(abs(x) >>r shift )
        vbit.s32    q15, q4, q3  // Restore sign of x with bit mask
    
    b .non_linear_loop

.q_shr_rounding_minus_inf:
    // return -(-x >>r shift)
    
        vneg.s32    q3, q8      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q8, q3      // Compute -(-x >>r shift)
    
        vneg.s32    q3, q9      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q9, q3      // Compute -(-x >>r shift)
    
        vneg.s32    q3, q10      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q10, q3      // Compute -(-x >>r shift)
    
        vneg.s32    q3, q11      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q11, q3      // Compute -(-x >>r shift)
    
        vneg.s32    q3, q12      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q12, q3      // Compute -(-x >>r shift)
    
        vneg.s32    q3, q13      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q13, q3      // Compute -(-x >>r shift)
    
        vneg.s32    q3, q14      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q14, q3      // Compute -(-x >>r shift)
    
        vneg.s32    q3, q15      // Compute -x
        vqrshl.s32  q3, q2          // Rounding shift (0.5 -> 1)
        vneg.s32    q15, q3      // Compute -(-x >>r shift)
    
    b .non_linear_loop

.q_shr_rounding_plus_inf:
    // return x >>r shift
    
        vqrshl.s32  q8, q2      // Rounding shift (0.5 -> 1)
    
        vqrshl.s32  q9, q2      // Rounding shift (0.5 -> 1)
    
        vqrshl.s32  q10, q2      // Rounding shift (0.5 -> 1)
    
        vqrshl.s32  q11, q2      // Rounding shift (0.5 -> 1)
    
        vqrshl.s32  q12, q2      // Rounding shift (0.5 -> 1)
    
        vqrshl.s32  q13, q2      // Rounding shift (0.5 -> 1)
    
        vqrshl.s32  q14, q2      // Rounding shift (0.5 -> 1)
    
        vqrshl.s32  q15, q2      // Rounding shift (0.5 -> 1)
    
    b .non_linear_loop

.q_shr_rounding_even:
    // If (x >> shift) is odd -> (x - 0) >>r shift
    // If (x >> shift) is even -> (x - 1) >>r shift
    
        vqshl.s32   q3, q8, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q8, q8, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q8, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q9, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q9, q9, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q9, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q10, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q10, q10, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q10, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q11, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q11, q11, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q11, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q12, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q12, q12, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q12, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q13, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q13, q13, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q13, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q14, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q14, q14, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q14, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q15, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x is odd
        vsub.s32    q5, q4, q1          // If (x >> shift) is odd 0 else -1
        vadd.s32   q15, q15, q5   // If (x >> shift) is odd (x - 0) else (x - 1)
        vqrshl.s32 q15, q2           // Rounding shift (0.5 -> 1)
    
    b .non_linear_loop

.q_shr_rounding_odd:
    // If (x >> shift) is even -> (x - 0) >>r shift
    // If (x >> shift) is odd -> (x - 1) >>r shift
    
        vqshl.s32   q3, q8, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q8, q8, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q8, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q9, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q9, q9, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q9, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q10, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q10, q10, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q10, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q11, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q11, q11, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q11, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q12, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q12, q12, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q12, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q13, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q13, q13, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q13, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q14, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q14, q14, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q14, q2           // Rounding shift (0.5 -> 1)
    
        vqshl.s32   q3, q15, q2      // Truncate shift (0.5 -> 0)
        vand.s32    q4, q3, q1          // Store if x >> shift is odd
        vneg.s32    q5, q4              // If x is odd -1 else 0
        vadd.s32   q15, q15, q5   // If x is odd (x - 1) else (x - 0)
        vqrshl.s32 q15, q2           // Rounding shift (0.5 -> 1)
    
    b .non_linear_loop


.store:
    // r3, r4, r5, r6 <- ptr, rsc, csc, size
    cmp     r6, #4
    beq     .store_strides_i32

    
        vmovn.s32 d16, q8
        vmovn.s16 d16, q8
    
        vmovn.s32 d18, q9
        vmovn.s16 d18, q9
    
        vmovn.s32 d20, q10
        vmovn.s16 d20, q10
    
        vmovn.s32 d22, q11
        vmovn.s16 d22, q11
    
        vmovn.s32 d24, q12
        vmovn.s16 d24, q12
    
        vmovn.s32 d26, q13
        vmovn.s16 d26, q13
    
        vmovn.s32 d28, q14
        vmovn.s16 d28, q14
    
        vmovn.s32 d30, q15
        vmovn.s16 d30, q15
    
    
        mov         r8, r3
        
            
            vst1.s8     d16[0], [ r8 ], r4
            vst1.s8     d16[1], [ r8 ], r4
            vst1.s8     d16[2], [ r8 ], r4
            vst1.s8     d16[3], [ r8 ], r4
        
            
            vst1.s8     d18[0], [ r8 ], r4
            vst1.s8     d18[1], [ r8 ], r4
            vst1.s8     d18[2], [ r8 ], r4
            vst1.s8     d18[3], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            
            vst1.s8     d20[0], [ r8 ], r4
            vst1.s8     d20[1], [ r8 ], r4
            vst1.s8     d20[2], [ r8 ], r4
            vst1.s8     d20[3], [ r8 ], r4
        
            
            vst1.s8     d22[0], [ r8 ], r4
            vst1.s8     d22[1], [ r8 ], r4
            vst1.s8     d22[2], [ r8 ], r4
            vst1.s8     d22[3], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            
            vst1.s8     d24[0], [ r8 ], r4
            vst1.s8     d24[1], [ r8 ], r4
            vst1.s8     d24[2], [ r8 ], r4
            vst1.s8     d24[3], [ r8 ], r4
        
            
            vst1.s8     d26[0], [ r8 ], r4
            vst1.s8     d26[1], [ r8 ], r4
            vst1.s8     d26[2], [ r8 ], r4
            vst1.s8     d26[3], [ r8 ], r4
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            
            vst1.s8     d28[0], [ r8 ], r4
            vst1.s8     d28[1], [ r8 ], r4
            vst1.s8     d28[2], [ r8 ], r4
            vst1.s8     d28[3], [ r8 ], r4
        
            
            vst1.s8     d30[0], [ r8 ], r4
            vst1.s8     d30[1], [ r8 ], r4
            vst1.s8     d30[2], [ r8 ], r4
            vst1.s8     d30[3], [ r8 ], r4
        
        
    

    b .non_linear_loop

.store_strides_i32:

    
        mov         r8, r3
        
            
                vst1.s32     d16[0], [ r8 ], r4
            
                vst1.s32     d16[1], [ r8 ], r4
            
        
            
                vst1.s32     d17[0], [ r8 ], r4
            
                vst1.s32     d17[1], [ r8 ], r4
            
        
            
                vst1.s32     d18[0], [ r8 ], r4
            
                vst1.s32     d18[1], [ r8 ], r4
            
        
            
                vst1.s32     d19[0], [ r8 ], r4
            
                vst1.s32     d19[1], [ r8 ], r4
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            
                vst1.s32     d20[0], [ r8 ], r4
            
                vst1.s32     d20[1], [ r8 ], r4
            
        
            
                vst1.s32     d21[0], [ r8 ], r4
            
                vst1.s32     d21[1], [ r8 ], r4
            
        
            
                vst1.s32     d22[0], [ r8 ], r4
            
                vst1.s32     d22[1], [ r8 ], r4
            
        
            
                vst1.s32     d23[0], [ r8 ], r4
            
                vst1.s32     d23[1], [ r8 ], r4
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            
                vst1.s32     d24[0], [ r8 ], r4
            
                vst1.s32     d24[1], [ r8 ], r4
            
        
            
                vst1.s32     d25[0], [ r8 ], r4
            
                vst1.s32     d25[1], [ r8 ], r4
            
        
            
                vst1.s32     d26[0], [ r8 ], r4
            
                vst1.s32     d26[1], [ r8 ], r4
            
        
            
                vst1.s32     d27[0], [ r8 ], r4
            
                vst1.s32     d27[1], [ r8 ], r4
            
        
        
            add r3, r3, r5
        
    
        mov         r8, r3
        
            
                vst1.s32     d28[0], [ r8 ], r4
            
                vst1.s32     d28[1], [ r8 ], r4
            
        
            
                vst1.s32     d29[0], [ r8 ], r4
            
                vst1.s32     d29[1], [ r8 ], r4
            
        
            
                vst1.s32     d30[0], [ r8 ], r4
            
                vst1.s32     d30[1], [ r8 ], r4
            
        
            
                vst1.s32     d31[0], [ r8 ], r4
            
                vst1.s32     d31[1], [ r8 ], r4
            
        
        
    

    b .non_linear_loop

.load_tile:
    vldmia          r3!, { q8-q15 }
    b .non_linear_loop

.return:
    vpop        { q4-q7 }
    pop         { r4-r12 }

    bx          lr



#define test 12+2\
             /3-2

#include "./assembler/res/test2.asm"

.this_is_a_sub_label:

label_test:

.this_is_a_sub_label:


addi $4, $4, test
addi $4, $4, test2_2

jeq $4, $5, .end

.end:
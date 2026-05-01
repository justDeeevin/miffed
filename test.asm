.data
  x: .asciiz "droddyrox\n"
  y: .word 0xff

.text
main: lw $t0, x
      add $t0, $t0, 1
      sw $t0, x
      mul $t0, $t0, 2
      sw $t0, x
      li $v0, 1
      move $a0, $t0
      syscall
      li $v0, 10
      syscall

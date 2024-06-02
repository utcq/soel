.global main
.section .data
.section .text
main:
    push R28
    push R29
    rcall .+0
    rcall .+0
    in R28, 61
    in R29, 62
    ldi R24, 5
    ldi R25, 0
    std Y+2, R25
    std Y+1, R24
    ldi R24, 8
    ldi R25, 0
    std Y+4, R25
    std Y+3, R24
    ldd R18, Y+1
    ldd R19, Y+2
    ldd R24, Y+3
    ldd R25, Y+4
    add R24, R18
    adc R25, R19
    pop R0
    pop R0
    pop R0
    pop R0
    pop R29
    pop R28
    ret

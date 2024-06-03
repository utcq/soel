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
    ldi R24, 3
    ldi R25, 0
    std Y+1, R24
    std Y+2, R25
    ldi R24, 4
    ldi R25, 0
    std Y+3, R24
    std Y+4, R25
    ldi R24, 3
    ldi R25, 0
    ldd R18, Y+3
    ldd R19, Y+4
    add R24, R18
    adc R25, R19
    ldi R18, 2
    ldi R19, 0
    ldd R16, Y+1
    ldd R17, Y+2
    add R18, R16
    adc R19, R17
    add R24, R18
    adc R25, R19
    pop R0
    pop R0
    pop R0
    pop R0
    pop R29
    pop R28
    ret

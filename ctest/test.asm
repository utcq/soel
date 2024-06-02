00000000 <main>:
   0:   cf 93           push    r28
   2:   df 93           push    r29
   8:   cd b7           in      r28, 0x3d       ; 61
   a:   de b7           in      r29, 0x3e       ; 62  SETUP STACK

   c:   85 e0           ldi     r24, 0x05       ; 5
   e:   90 e0           ldi     r25, 0x00       ; 0
  10:   9a 83           std     Y+2, r25        ; 0x02
  12:   89 83           std     Y+1, r24        ; 0x01 FIRST VARIABLE DECLARATION

  14:   88 e0           ldi     r24, 0x08       ; 8
  16:   90 e0           ldi     r25, 0x00       ; 0
  18:   9c 83           std     Y+4, r25        ; 0x04
  1a:   8b 83           std     Y+3, r24        ; 0x03 SECOND VARIABLE DECLARATION

  1c:   80 e0           ldi     r24, 0x00       ; 0
  1e:   90 e0           ldi     r25, 0x00       ; 0 SETTING RETURN VALUE


  20:   0f 90           pop     r0
  22:   0f 90           pop     r0
  24:   0f 90           pop     r0
  26:   0f 90           pop     r0              ; CLEANING STACK (4 BYTES)


  28:   df 91           pop     r29
  2a:   cf 91           pop     r28
  2c:   08 95           ret                     ; EPILOGUE (STACK RESTORE AND RETURN)


; int main() {
;   int x = 5;
;   int y = 8;  
;   return 0;
;}
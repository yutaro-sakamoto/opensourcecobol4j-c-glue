           identification division.
              program-id.  prog.
           data division.
           working-storage section.
           01 a pic 9(5) usage binary value 12345.
           01 b pic 9(5) usage binary value 1024.
           01 i pic 9(5) usage binary value 22.
           01 small-data. 
             03 first-name pic x(10) value "Taro" & X'00'.
             03 second-name pic x(10) value "Yamada" & X'00'.
           procedure division.
                call "init" USING a b.
                call "destroy" USING small-data i.

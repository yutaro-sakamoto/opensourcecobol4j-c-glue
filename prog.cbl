           identification division.
              program-id.  prog.
           data division.
           working-storage section.
           01 a pic 9(5) usage binary value 1.
           01 b pic 9(5) usage binary value 1024.
           procedure division.
                call "init" USING a b.
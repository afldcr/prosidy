## This Source Code Form is subject to the terms of the Mozilla Public
## License, v. 2.0. If a copy of the MPL was not distributed with this
## file, You can obtain one at https://mozilla.org/MPL/2.0/.

003 - Tags
---

#-simple

#-props[foo, bar='baz']

#-nested{Content!}

#-multiline:
    Some #em[ru="еще"]{more} content!
#:

#-propsnested[foo, bar='baz']{Even #em[es="más"]{more} content!}

#-propsmultiline[qux, baz='foo']:
    #-nestedinmultiline:
        Ok then
    #:
#:

#+lit:end
#this{isn't} valid at all!
#:
#:
#:
#:end

#+lit[flag, withprops='true']:
    this literal has properties!
#:

#-content:
    #+lit:
        Literals can be nested!
    #:
#:

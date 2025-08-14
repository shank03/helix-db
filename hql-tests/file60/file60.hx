N::Chapter {
    title:String,
    isRoot:Boolean,
    content:String
}

QUERY getAllRootChapters() =>
    // ok when using helix check, but error occurs when using helix deploy
    //rootChapters <- N<Chapter>::WHERE(_::{isRoot})

    //Error compiling queries
    rootChapters <- N<Chapter>::WHERE(_::{isRoot}::EQ(true))
    RETURN rootChapters
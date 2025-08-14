N::Chapter {
    title:String,
    isRoot:Boolean,
    isLeaf:Boolean,
    EmbeddingStatus:U32,
    content:String
}

E::Contain{
    From: Chapter, 
    To:Chapter,
    Properties: {
    }
}

QUERY createChapter(
    title:String,
    isRoot:Boolean,
    isLeaf:Boolean,
    content:String
) =>
    chapter <- AddN<Chapter>({
        title:title,
        isRoot:false,
        isLeaf:true,
        EmbeddingStatus:0,
        content:content
    })
    RETURN chapter

QUERY setContainRelationship(parentChapter_id:ID, childChapter_id:ID)=>
    parentChapter <- N<Chapter>(parentChapter_id)::UPDATE({isLeaf:false})
    //remove the UPDATE above, everything will be fine 
    childChapter <- N<Chapter>(childChapter_id)
    contain <- AddE<Contain>()::From(parentChapter)::To(childChapter)
    
    RETURN parentChapter,childChapter,contain
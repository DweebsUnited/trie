struct Node {
    c: char,
    used: u64,
    eowc: u64,

    children: Vec<Node>,
}

pub struct Trie {
    root: Node,
}


impl Node {

    // Create
    fn new( c: char ) -> Node {
        Node::new_with_stats( c, 0, 0 )
    }

    // Create with existing values
    fn new_with_stats( c: char, used: u64, eowc: u64 ) -> Node {
        Node {
            c: c,
            used: used,
            eowc: eowc,
            children: Vec::new( )
        }
    }

    // Query for a child
    fn get_child( &self, c: char ) -> Option<&Node> {

        if let Ok( cdx ) = self.children.binary_search_by_key( &c, | n: &Node | n.c ) {
            Some( &self.children[ cdx ] )
        } else {
            None
        }

    }

    // Get child if exists, or add a new one
    fn get_or_add_child( &mut self, c: char ) -> &mut Node {

        let cdx_opt = self.children.binary_search_by_key( &c, | n: &Node | n.c );

        match cdx_opt {
            Ok( cdx ) => {
                self.children.get_mut( cdx ).expect( "Couldn't query the child we just found..." )
            },
            Err( cdx ) => {
                // Insert sorted
                self.children.insert( cdx, Node::new( c ) );
                // Have to query it to return it
                self.children.get_mut( cdx ).expect( "Couldn't query the child we just inserted..." )
            }
        }

    }

    // Add a new word
    fn add( &mut self, word: &str ) {

        self.used += 1;

        if word.len( ) == 0 {
            self.eowc += 1;
            return;
        }

        let c: char = word.chars( ).nth( 0 ).unwrap( );
        let child: &mut Node = self.get_or_add_child( c );
        child.add( &word[ 1.. ] );

    }

    // Recursively query a word
    fn query( &self, word: &str ) -> Option<u64> {

        if word.len( ) == 0 {
            return Some( self.eowc );
        }

        let c = word.chars( ).nth( 0 ).unwrap( );

        if let Some( child ) = self.get_child( c ) {
            child.query( &word[ 1.. ] )
        } else {
            None
        }

    }

}

impl Trie {

    // Create a Trie
    pub fn new( ) -> Trie {
        Trie {
            root: Node::new( '\0' )
        }
    }

    // Add new word
    pub fn add( &mut self, word: &str ) {

        self.root.add( word );

    }

    // Query a word starting at root -> eowc
    pub fn query( &self, word: &str ) -> Option<u64> {

        self.root.query( word )

    }

    // TODO: walk -> Takes a fn at each step with whether to descend or not

}

pub mod io {

    use crate::{ Trie, Node };

    use std::fs::File;
    use std::path::Path;
    use std::io::{ self, BufRead };

    pub fn from_wordlist<P: AsRef<Path>>( path: P ) -> Result<Trie, io::Error> {

        let f: File = File::open( path )?;
        let lines: io::Lines<io::BufReader<File>> = io::BufReader::new( f ).lines( );

        let mut t: Trie = Trie::new( );

        for line in lines {

            if let Ok( word ) = line {

                t.add( &word );

            }

        }

        Ok( t )

    }

    pub fn write_text<P: AsRef<Path>>( t: &Trie, path: P ) -> Result<(), io::Error> {

        let f: File = File::create( path )?;
        let mut w: io::BufWriter<File> = io::BufWriter::new( f );

        t.write( &mut w )

    }

    impl Trie {

        fn write<W: io::Write>( &self, w: &mut W ) -> Result<(), io::Error> {

            self.root.write( w )?;
            w.flush( )

        }

    }

    impl Node {

        fn write<W: io::Write>( &self, w: &mut W ) -> Result<(), io::Error> {

            write!( w, "{}\x1F{}\x1F{}\x1F{}", self.c, self.used, self.eowc, self.children.len( ) )?;

            for child in self.children {

                child.write( w )?

            }

        }

    }

}
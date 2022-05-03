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
            if self.eowc > 0 {
                return Some( self.eowc );
            } else {
                return None;
            }
        }

        let c = word.chars( ).nth( 0 ).unwrap( );

        if let Some( child ) = self.get_child( c ) {
            child.query( &word[ 1.. ] )
        } else {
            None
        }

    }

    // Walking functions!
    // Private, but I don't really like having to specify the add value every time...
    fn walk<P: FnMut( &str, u64, u64 ) -> bool>( &self, s: &mut String, pred: &mut P, add: bool ) {

        if add {

            s.push( self.c );

            if ! pred( &s, self.used, self.eowc ) {
                s.pop( );
                return;
            }

        }

        for n in &self.children {

            n.walk( s, pred, true );

        }

        if add {
            s.pop( );
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

    pub fn walk<P: FnMut( &str, u64, u64 ) -> bool>( &self, pred: &mut P ) {

        let mut word: String = String::new( );
        self.root.walk( &mut word, pred, false );

    }

}

pub mod io {

    use crate::{ Trie, Node };

    use std::fs::File;
    use std::path::Path;
    use std::io::{ self, BufRead, Result, Error, ErrorKind };
    use std::iter;

    pub fn from_wordlist<P: AsRef<Path>>( path: P ) -> Result<Trie> {

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

    pub fn from_wordlist_if<P: AsRef<Path>, B: Fn( &str ) -> bool>( path: P, pred: B ) -> Result<Trie> {

        let f: File = File::open( path )?;
        let lines: io::Lines<io::BufReader<File>> = io::BufReader::new( f ).lines( );

        let mut t: Trie = Trie::new( );

        for line in lines {

            if let Ok( word ) = line {

                if pred( &word ) {

                    t.add( &word );

                }

            }

        }

        Ok( t )

    }

    pub fn write_text<P: AsRef<Path>>( t: &Trie, path: P ) -> Result<()> {

        let f: File = File::create( path )?;
        let mut w: io::BufWriter<File> = io::BufWriter::new( f );

        t.write( &mut w )

    }

    pub fn read_text<P: AsRef<Path>>( path: P ) -> Result<Trie> {

        let f: File = File::open( path )?;
        let lines: io::Lines<io::BufReader<File>> = io::BufReader::new( f ).lines( );

        let mut t: Trie = Trie::new( );

        t.read( lines )?;

        Ok( t )

    }

    impl Trie {

        fn write<W: io::Write>( &self, w: &mut W ) -> Result<()> {

            self.root.write( w )?;
            w.flush( )

        }

        fn read<L>( &mut self, mut lines: L ) -> Result<()>
        where L: iter::Iterator<Item = Result<String>>,
        {

            self.root.read( &mut lines )

        }

    }

    impl Node {

        fn write<W: io::Write>( &self, w: &mut W ) -> Result<()> {

            write!( w, "{}\x1F{}\x1F{}\x1F{}\n", self.c, self.used, self.eowc, self.children.len( ) )?;

            for child in &self.children {

                child.write( w )?;

            }

            Ok(())

        }

        fn read<L>( &mut self, lines: &mut L ) -> Result<()>
        where L: iter::Iterator<Item = Result<String>>,
        {

            // Get root stats then start loop
            let line: String = lines.next( ).ok_or( Error::new( ErrorKind::Other, "No more lines while building node!" ) )??;
            let mut stats: std::str::Split<char> = line.trim( ).split( '\x1F' );

            let c: &str = stats.next( ).ok_or( Error::new( ErrorKind::Other, "Failed to get character from line split" ) )?;
            if c.len( ) != 1 {
                return Err( Error::new( ErrorKind::Other, "Parsed node character is wrong length!" ) );
            } else {
                self.c = c.chars( ).nth( 0 ).unwrap( );
            }

            let u: &str = stats.next( ).ok_or( Error::new( ErrorKind::Other, "Failed to get used from line split" ) )?;
            if let Ok( u ) = u.parse::<u64>( ) {
                self.used = u;
            } else {
                return Err( Error::new( ErrorKind::Other, "Could not parse used value to u64!" ) );
            }

            let e: &str = stats.next( ).ok_or( Error::new( ErrorKind::Other, "Failed to get eowc from line split" ) )?;
            if let Ok( e ) = e.parse::<u64>( ) {
                self.eowc = e;
            } else {
                return Err( Error::new( ErrorKind::Other, "Could not parse eowc value to u64!" ) );
            }

            let c_cnt_str: &str = stats.next( ).ok_or( Error::new( ErrorKind::Other, "Failed to get child count from line split" ) )?;
            let c_cnt: u64;
            if let Ok( c_cnt_prs ) = c_cnt_str.parse::<u64>( ) {
                c_cnt = c_cnt_prs;
            } else {
                return Err( Error::new( ErrorKind::Other, "Could not parse child count value to u64!" ) );
            }

            for _ in 0..c_cnt {

                let mut c: Node = Node::new( '\0' );
                c.read( lines )?;
                self.children.push( c );

            }

            Ok(())

        }

    }

}
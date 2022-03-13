use std::collections::LinkedList;
use parsing_lexer::tokenizer::{Token, TokenData, Tokenizer, TokenType};

pub struct HighlighterTokenizer<'input>{
    tokenizer: Tokenizer<'input>,
    err: LinkedList<Tok>,
    out: LinkedList<Tok>,
    pos: usize,
}

impl<'input> HighlighterTokenizer<'input>{
    pub fn new(tokenizer: Tokenizer<'input>) -> Self{
        HighlighterTokenizer{
            tokenizer: tokenizer
                .include_whitespace(true)
                .include_documentation(true)
                .include_comments(true),
            err: LinkedList::new(),
            out: LinkedList::new(),
            pos: 0,
        }
    }
    pub fn t(&mut self) -> &mut Tokenizer<'input>{
        &mut self.tokenizer
    }

    fn add_to_out(&mut self,tok: Tok){
        insert_into_list(&mut self.out, tok);
    }

    fn add_to_err(&mut self,tok: Tok){
        insert_into_list(&mut self.err, tok);
    }
}

fn insert_into_list(list: &mut LinkedList<Tok>, item: Tok){
    if get_data(&item).size == 0{
        return;
    }

    let mut tmp = LinkedList::new();

    'loo:
    loop{
        match list.pop_front(){
            None => {
                tmp.push_back(item);
                break 'loo;
            }
            Some(val) => {
                if get_data(&val).index > get_data(&item).index{
                    tmp.push_back(item);
                    tmp.push_back(val);
                    break 'loo;
                }else{
                    tmp.push_back(val);
                }
            }
        }
    }
    loop{
        match tmp.pop_back(){
            None => {
                return;
            }
            Some(val) => {
                list.push_front(val);
            }
        }
    }
}

fn get_data(tok:&Tok) -> &TokenData{
    match tok{
        Ok(val) => {
            &val.0
        }
        Err(val) => {
            &val.0
        }
    }
}

type Tok = Result<(TokenData, Token), (TokenData, Token, TokenType)>;

fn ok_from_tok(token: Token) -> Option<Tok> {
    Option::Some(Result::Ok((*token.get_token_data(), token)))
}
fn ok_from_tok_data(t_data: TokenData, token: Token) -> Option<Tok> {
    Option::Some(Result::Ok((t_data, token)))
}

fn err_from_tok(token: Token) -> Tok {
    let tmp = token.get_token_type().clone();
    Result::Err((token.t_data, token, tmp))
}

#[derive(Debug)]
enum Pos {
    Behind,
    Inside,
    Ahead,
}

fn get_pos(t1: &TokenData, t2: &TokenData) -> (Pos, Pos){
    ({
         if t1.get_index() < t2.get_index(){
             Pos::Behind
         }else if t1.get_index() > t2.get_index() + t2.get_size(){
             Pos::Ahead
         }else{
             Pos::Inside
         }
     },{
         if t1.get_index() + t1.get_size() - 1 < t2.get_index(){
             Pos::Behind
         }else if t1.get_index() + t2.get_size() - 1 > t2.get_index() + t2.get_size(){
             Pos::Ahead
         }else{
             Pos::Inside
         }
     })
}

impl<'input> Iterator for HighlighterTokenizer<'input> {
    type Item = Result<Tok, (usize, usize)>;

    fn next(&mut self) -> Option<Self::Item> {

        macro_rules! verify_item{
            ($item: expr) => {
                {
                    let tok = $item;
                    let data = *get_data(&tok);
                    //println!("{}, {}", self.pos, data.get_real_index());
                    if self.pos < data.get_real_index(){
                        self.add_to_out(tok);
                        let old = self.pos;
                        self.pos = data.get_real_index();
                        return Option::Some(Result::Err::<Tok, (usize, usize)>((old, data.get_real_index() - old)));
                    }else if self.pos > data.get_real_index() {

                    }else{
                        self.pos = data.get_real_index() + data.get_real_size();
                        return Option::Some(Result::Ok::<Tok, (usize, usize)>(tok));
                    }
                }
            }
        }
        loop{
            let tok = match self.out.pop_front() {
                None => {
                    match self.tokenizer.next(){
                        None => Option::None,
                        Some(val) => ok_from_tok(val),
                    }
                }
                val @Some(_) => {
                    val
                }
            };
            match tok{
                None => {
                    match self.err.pop_front(){
                        None => return Option::None,
                        Some(val) => verify_item!(val),
                    }
                    //if self.err.is_empty(){
                    //    return Option::None;
                    //}else{
                    //    return self.err.pop_front().unwrap();
                    //}
                }
                Some(val) => {

                    match val{
                        Err(val) => {
                            let val = Result::Err(val);
                            verify_item!(val);
                        }
                        Ok(val) => {
                            match val.1{
                                val @ Token{t_type: TokenType::ERROR(_), ..} => {
                                    self.add_to_err(err_from_tok(val));
                                }
                                _ => {
                                    if self.err.is_empty() {
                                        let val = Result::Ok(val);
                                        verify_item!(val);
                                    }else{
                                        let peek = self.err.front().unwrap();

                                        let pos = get_pos( get_data(&peek), &val.0);

                                        //println!("{:?}", pos);
                                        match pos{
                                            (Pos::Behind, Pos::Behind) => {
                                                self.add_to_out(Result::Ok(val));
                                               verify_item!(self.err.pop_front().unwrap());
                                            }
                                            //(Pos::Behind, Pos::Inside) => {
                                            //    //println!("Behind Inside");
                                            //    self.add_to_out(Result::Ok(val));
                                            //    return err_from_tok(self.err.pop_front().unwrap());
                                            //}
                                            (Pos::Inside, Pos::Inside) => {
                                                //println!("Inside Inside");
                                                let mut front = val.0;
                                                let mut middle = self.err.pop_front().unwrap();
                                                let mut back = val.0;

                                                front.size = get_data(&middle).get_index() - front.index;
                                                front.size_real = get_data(&middle).get_real_index() - front.index_real;


                                                match &mut middle{
                                                    Ok(_) => {}
                                                    Err( err) => {
                                                        err.2 = val.1.t_type.clone();
                                                    }
                                                }

                                                back.index = get_data(&middle).get_index() + get_data(&middle).get_size();
                                                back.index_real = get_data(&middle).get_real_index() + get_data(&middle).get_real_size();
                                                back.size = (val.0.index + val.0.size) - back.index;
                                                back.size_real = (val.0.index_real + val.0.size_real) - back.index_real;

                                                //println!("front{:?} back{:?}", front, back);

                                                self.add_to_out(ok_from_tok_data(front,val.1.clone()).unwrap());
                                                self.add_to_out(middle);
                                                self.add_to_out(ok_from_tok_data(back,val.1).unwrap());
                                                //return err_from_tok(self.err.pop_front().unwrap());
                                            }
                                            //(Pos::Inside, Pos::Behind) => {
                                            //    //println!("Inside Behind");
                                            //    self.add_to_out(Result::Ok(val));
                                            //    return err_from_tok(self.err.pop_front().unwrap());
                                            //}
                                            (Pos::Ahead, Pos::Ahead) => {
                                                let val = Result::Ok(val);
                                                verify_item!(val);
                                            }
                                            _ => {
                                                //panic!("impossible?? {:?}", pos);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


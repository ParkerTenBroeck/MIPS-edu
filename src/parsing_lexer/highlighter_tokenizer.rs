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
            tokenizer: tokenizer.include_whitespace(true),
            err: LinkedList::new(),
            out: LinkedList::new(),
            pos: 0,
        }
    }
    pub fn t(&mut self) -> &mut Tokenizer<'input>{
        &mut self.tokenizer
    }

    fn add_to_out(&mut self,tok: Tok){
        let mut tmp = LinkedList::new();

        'loo:
        loop{
            match self.out.pop_front(){
                None => {
                    tmp.push_back(tok);
                    break 'loo;
                }
                Some(val) => {
                    if get_data(&val).index > get_data(&tok).index{
                        tmp.push_back(tok);
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
                    self.out.push_front(val);
                }
            }
        }
    }

    fn add_to_err(&mut self,tok: Tok){
        let mut tmp = LinkedList::new();

        'loo:
        loop{
            match self.err.pop_front(){
                None => {
                    tmp.push_back(tok);
                    break 'loo;
                }
                Some(val) => {
                    if get_data(&val).index > get_data(&tok).index{
                        tmp.push_back(tok);
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
                    self.err.push_front(val);
                }
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
fn err_from_tok_type(t_type: TokenType, token: Token) -> Tok {
    Result::Err((token.t_data, token, t_type))
}

impl<'input> Iterator for HighlighterTokenizer<'input> {
    type Item = Tok;

    fn next(&mut self) -> Option<Self::Item> {

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
                        Some(val) => return Option::Some(val),
                    }
                    //if self.err.is_empty(){
                    //    return Option::None;
                    //}else{
                    //    return self.err.pop_front().unwrap();
                    //}
                }
                Some(val) => {

                    match val{
                        Err(val) => return Option::Some(Result::Err(val)),

                        Ok(val) => {
                            match val.1{
                                val @ Token{t_type: TokenType::ERROR(_), ..} => {
                                    self.add_to_err(err_from_tok(val));
                                }
                                _ => {
                                    if self.err.is_empty() {
                                        return Option::Some(Result::Ok(val));
                                    }else{
                                        let peek = self.err.front().unwrap();
                                        #[derive(Debug)]
                                        enum Pos {
                                            Behind,
                                            Inside,
                                            Ahead,
                                        }
                                        let pos = {
                                            ({
                                                if get_data(&peek).get_index() <= val.0.get_index(){
                                                    Pos::Behind
                                                }else if get_data(&peek).get_index() > val.0.get_index() + val.0.get_size(){
                                                    Pos::Ahead
                                                }else{
                                                    Pos::Inside
                                                }
                                             },{
                                                if get_data(&peek).get_index() + get_data(&peek).get_size() <= val.0.get_index(){
                                                    Pos::Behind
                                                }else if get_data(&peek).get_index() + get_data(&peek).get_size() > val.0.get_index() + val.0.get_size(){
                                                    Pos::Ahead
                                                }else{
                                                    Pos::Inside
                                                }
                                            })
                                        };

                                        println!("{:?}", pos);
                                        match pos{
                                            (Pos::Behind, Pos::Behind) => {
                                                self.add_to_out(Result::Ok(val));
                                                return self.err.pop_front();
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
                                                    Err( val) => {
                                                        val.2 = val.1.t_type.clone();
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
                                                return Option::Some(Result::Ok(val));
                                            }
                                            _ => {
                                                panic!("impossible?? {:?}", pos);
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


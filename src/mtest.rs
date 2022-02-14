macro_rules! zoom_and_enhance {
    (struct $name:ident { $($fname:ident : $ftype:ty),* }) => {
        struct $name {
            $($fname : $ftype),*
        }

        impl $name {
            fn field_names() -> &'static [&'static str] {
                static NAMES: &'static [&'static str] = &[$(stringify!($fname)),*];
                NAMES
            }
        }
    }
}

zoom_and_enhance!{
struct Export {
    first_name: String,
    last_name: String,
    gender: String,
    date_of_birth: String,
    address: String
}
}

macro_rules! mixed_rules {
    () => {};
    (trace $name:ident; $($tail:tt)*) => {
        {
            println!(concat!(stringify!($name), " = {:?}"), $name);
            mixed_rules!($($tail)*);
        }
    };
    (trace $name:ident = $init:expr; $($tail:tt)*) => {
        {
            let $name = $init;
            println!(concat!(stringify!($name), " = {:?}"), $name);
            mixed_rules!($($tail)*);
        }
    };
}
macro_rules! sum {
    ($base:expr) => { $base };
    ($a:expr, $($rest:expr),+) => {
        [$a, $($rest),+]
    };
}

// + sum2!($($rest),+)
macro_rules! sum2 {
    ($base:expr) => { $base };
    ($a:expr,  $($rest:expr),+) => {
        match 1{
            [$a, $($rest),+] => {}
            _ => {}
        }
    };
}

macro_rules! sum3 {
    ($obj:ident, $($rest:pat ),+, $inside:block) => {
        sum3!(0, $obj, $($rest),+, $inside);
    };
    ($num:expr, $obj:ident, $a:pat , $($rest:pat ),+, $inside:block) => {
        if let Option::Some($a) = $obj.get_mut($num){
            sum3!($num + 1, $obj, $($rest),+, $inside);
        }else if $obj.len() == $num{

        }else{
            return;
        }

    };
    ($num:expr, $obj:ident, $base:pat , $inside:block) => {
        if $obj.len() == $num + 1 {
            if let Option::Some($base) = $obj.get_mut($num){
                $inside
            }else{

            }
        }else{}

    };
}

macro_rules! test{
    ($name:ident) => {
      fn $name(){

      }
    };
}

enum Test{
    NOTHING,
    Val(i32)
}

test!(thisisaname);

fn test(){
    let test:&mut [Test] = Vec::new().as_mut_slice();

    sum3!(test,Test::NOTHING,Test::NOTHING,Test::Val(test), {
        print(test);
    });
}

fn print(num: &mut i32){
    println!("{}", num)
}

pub(super) fn add_sub_3(stack_slice: &mut [NonTerminal]) -> ReducerResponse {
    if let Option::Some(NonTerminal::AddSub(left)) = stack_slice.get_mut(0) {
        if let Option::Some(NonTerminal::Terminal(
                                operator @ Token {
                                    t_type: TokenType::Plus | TokenType::Minus,
                                    ..
                                },
                            )) = stack_slice.get_mut(0 + 1)
        {
            if stack_slice.len() == 0 + 1 + 1 + 1 {
                if let Option::Some(NonTerminal::AddSub(right)) =
                stack_slice.get_mut(0 + 1 + 1)
                {
                    match stack_slice {
                        [NonTerminal::AddSub(left), NonTerminal::Terminal(
                            operator @ Token {
                                t_type: TokenType::Plus | TokenType::Minus,
                                ..
                            },
                        ), NonTerminal::AddSub(right)] => {
                            return {
                                ReducerResponse::Reduce(NonTerminal::AddSub(Option::Some(
                                    Box::new(BinaryOperator {
                                        left_size: mem::take(left).expect(""),
                                        operator: steal(operator),
                                        right_size: mem::take(right).expect(""),
                                    }),
                                )))
                            };
                        }
                        _ => {
                            return ReducerResponse::NoMatch;
                        }
                    }
                } else {
                    return ReducerResponse::NoMatch;
                }
            } else {
                return ReducerResponse::NoMatch;
            };
        } else if stack_slice.len() == 0 + 1 {
            return ReducerResponse::PossibleMatch;
        } else {
            return ReducerResponse::NoMatch;
        };
    } else if stack_slice.len() == 0 {
        return ReducerResponse::PossibleMatch;
    } else {
        return ReducerResponse::NoMatch;
    };
}
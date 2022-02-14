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
        }else{

        }

    };
    ($num:expr, $obj:ident, $base:pat , $inside:block) => {
        if $obj.len() == $num + 1 {
            if let Option::Some($base) = $obj.get_mut($num){
                $inside
            }else{

            }
        }
    };
}

enum Test{
    NOTHING,
    Val(i32)
}

fn test(){
    let test:&mut [Test] = Vec::new().as_mut_slice();

    sum3!(test,Test::NOTHING,Test::NOTHING,Test::Val(test), {
        print(test);
    });
}

fn print(num: &mut i32){
    println!("{}", num)
}
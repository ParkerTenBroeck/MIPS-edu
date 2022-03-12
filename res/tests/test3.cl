i32 Test(i32 i1, ...){
    i32 a = 44;
    i32 b=55;
    i32 c=b;
    bool comp = a < b;
    if(comp==true){
        return b - a;
    }else{
        return a + b;
    }
}

i32 main2(){
    i32 a = 1;
    i32 b = 2;
    char* testString = 'as\nd\'';
    return Test() + a + b;
}
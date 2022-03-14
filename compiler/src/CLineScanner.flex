
%%

%public

%function next_token

%unicode

%line
%column
%char

%{
%}

/* main character classes */
LineTerminator = \r|\n|\r\n
InputCharacter = [^\r\n]

WhiteSpace = {LineTerminator} | [ \t\f]

/* comments */
Comment = {TraditionalComment} |
          {DocumentationComment}

TraditionalComment = "/*" [^*] ~"*/" | "/*" "*"+ "/"
EndOfLineComment = "//" | ";" {InputCharacter}* {LineTerminator}?
DocumentationComment = "/*" "*"+ [^/*] ~"*/"

OctDigit          = [0-7]
/* identifiers */
ALetter = [a-zA-Z_$\.]
ALetterDigit = [a-zA-Z_$0-9\.]
Identifier = {ALetter}{ALetterDigit}*

/* integer literals */
DecIntegerLiteral = [0-9][0-9_]*
DecIntegerI8Literal = {DecIntegerLiteral} i8
DecIntegerI16Literal = {DecIntegerLiteral} i16
DecIntegerI32Literal = {DecIntegerLiteral} i32
DecIntegerI64Literal = {DecIntegerLiteral} i64
DecIntegerU8Literal = {DecIntegerLiteral} u8
DecIntegerU16Literal = {DecIntegerLiteral} u16
DecIntegerU32Literal = {DecIntegerLiteral} u32
DecIntegerU64Literal = {DecIntegerLiteral} u64

HexDigit          = [0-9a-fA-F_]
HexIntegerLiteral = 0x {HexDigit}
HexIntegerI8Literal = {HexIntegerLiteral} i8
HexIntegerI16Literal = {HexIntegerLiteral} i16
HexIntegerI32Literal = {HexIntegerLiteral} i32
HexIntegerI64Literal = {HexIntegerLiteral} i64
HexIntegerU8Literal = {HexIntegerLiteral} u8
HexIntegerU16Literal = {HexIntegerLiteral} u16
HexIntegerU32Literal = {HexIntegerLiteral} u32
HexIntegerU64Literal = {HexIntegerLiteral} u64

BinaryDigit          = [0-1]
BinaryIntegerLiteral = 0b {BinaryDigit}
BinIntegerI8Literal = {BinaryIntegerLiteral} i8
BinIntegerI16Literal = {BinaryIntegerLiteral} i16
BinIntegerI32Literal = {BinaryIntegerLiteral} i32
BinIntegerI64Literal = {BinaryIntegerLiteral} i64
BinIntegerU8Literal = {BinaryIntegerLiteral} u8
BinIntegerU16Literal = {BinaryIntegerLiteral} u16
BinIntegerU32Literal = {BinaryIntegerLiteral} u32
BinIntegerU64Literal = {BinaryIntegerLiteral} u64


/* floating point literals */
FLit1    = [0-9]+ \. [0-9]*
FLit2    = \. [0-9]+
FLit3    = [0-9]+
Exponent = [eE] [+-]? [0-9]+

FloatF32Literal  = ({FLit1}|{FLit2}|{FLit3}) {Exponent}? [f32]{0,1}
FloatF64Literal = ({FLit1}|{FLit2}|{FLit3}) {Exponent}? f64

/* string and character literals */
StringCharacter = [^\r\n\"\\]
SingleCharacter = [^\r\n\'\\]

%state STRING, CHARLITERAL

%%

<YYINITIAL> {

  /* keywords */
  "void"                        { return symbol(VoidKeyword); }
  "struct"                      { return symbol(StructKeyword); }
  "asm"                         { return symbol(AsmKeyword); }
  "const"                       { return symbol(ConstKeyword); }
  "static"                      { return symbol(StaticKeyword); }
  "sizeof"                      { return symbol(SizeofKeyword); }
  "enum"                        { return symbol(EnumKeyword); }
  "if"                          { return symbol(IfKeyword); }
  "else"                        { return symbol(ElseKeyword); }
  "while"                       { return symbol(WhileKeyword); }
  "do"                          { return symbol(DoKeyword); }
  "for"                         { return symbol(ForKeyword); }
  "return"                      { return symbol(ReturnKeyword); }
  "break"                       { return symbol(BreakKeyword); }
  "switch"                      { return symbol(SwitchKeyword); }
  "case"                        { return symbol(CaseKeyword); }
  "goto"                        { return symbol(GotoKeyword); }
  "restrict"                    { return symbol(RestrictKeyword); }
  "i8"                          { return symbol(I8Keyword); }
  "i16"                         { return symbol(I16Keyword); }
  "i32"                         { return symbol(I32Keyword); }
  "u8"                          { return symbol(U8Keyword); }
  "u16"                         { return symbol(U16Keyword); }
  "u32"                         { return symbol(U32Keyword); }
  "char"                        { return symbol(CharKeyword); }
  "bool"                        { return symbol(BoolKeyword); }

  /* boolean literals */
  "true"                         { return symbol(BOOLEAN_LITERAL, true); }
  "TRUE"                         { return symbol(BOOLEAN_LITERAL, true); }
  "false"                        { return symbol(BOOLEAN_LITERAL, false); }
  "FALSE"                        { return symbol(BOOLEAN_LITERAL, false); }
  /* number literals */

  /* null literal */
  //"null"                         { return symbol(NULL_LITERAL); }


  /* separators */
  "("                            { return symbol(LPAREN); }
  ")"                            { return symbol(RPAREN); }
  "{"                            { return symbol(LBRACE); }
  "}"                            { return symbol(RBRACE); }
  "["                            { return symbol(LBRACK); }
  "]"                            { return symbol(RBRACK); }
  ";"                            { return symbol(SEMICOLON); }
  ","                            { return symbol(COMMA); }
  "."                            { return symbol(DOT); }
  "..."                          { return symbol(DOTDOTDOT); }
  "->"                           { return symbol(ARROR); }

  /* operators */
  "="                            { return symbol(EQ); }
  ">"                            { return symbol(GT); }
  "<"                            { return symbol(LT); }
  "!"                            { return symbol(NOT); }
  "~"                            { return symbol(COMP); }
  "?"                            { return symbol(QUESTION); }
  ":"                            { return symbol(COLON); }
  "=="                           { return symbol(EQEQ); }
  "<="                           { return symbol(LTEQ); }
  ">="                           { return symbol(GTEQ); }
  "!="                           { return symbol(NOTEQ); }
  "&&"                           { return symbol(ANDAND); }
  "||"                           { return symbol(OROR); }
  "+"                            { return symbol(PLUS); }
  "-"                            { return symbol(MINUS); }
  "*"                            { return symbol(MULT); }
  "/"                            { return symbol(DIV); }
//  "\\"                           { return symbol(BACKSLASH); }
  "&"                            { return symbol(AND); }
  "|"                            { return symbol(OR); }
  "^"                            { return symbol(XOR); }
  "%"                            { return symbol(MOD); }
  "<<"                           { return symbol(LSHIFT); }
  ">>"                           { return symbol(RSHIFT); }
  "+="                           { return symbol(PLUSEQ); }
  "-="                           { return symbol(MINUSEQ); }
  "*="                           { return symbol(MULTEQ); }
  "/="                           { return symbol(DIVEQ); }
  "&="                           { return symbol(ANDEQ); }
  "|="                           { return symbol(OREQ); }
  "^="                           { return symbol(XOREQ); }
  "%="                           { return symbol(MODEQ); }
  "<<="                          { return symbol(LSHIFTEQ); }
  ">>="                          { return symbol(RSHIFTEQ); }

  /* string literal */
  \"                             { yybegin(STRING); string.setLength(0); columnStringStart = yycolumn; charStringStart = yychar;}

  /* character literal */
  \'                             { yybegin(CHARLITERAL); }
  /* numeric literals */

  /* This is matched together with the minus, because the number is too big to
     be represented by a positive integer. */
  "-2147483648"                  { return symbol(INTEGER_LITERAL, Integer.valueOf(Integer.MIN_VALUE)); }

  {DecIntegerLiteral}            { return symbol(INTEGER_LITERAL, Integer.valueOf(yytext())); }
  //{DecLongLiteral}               { return symbol(INTEGER_LITERAL, new Long(yytext().substring(0,yylength()-1))); }

  {HexIntegerLiteral}            { return symbol(INTEGER_LITERAL, Integer.valueOf((int) parseLong(2, yylength(), 16))); }
  //{HexLongLiteral}               { return symbol(INTEGER_LITERAL, new Long(parseLong(2, yylength()-1, 16))); }

//  {OctIntegerLiteral}            { return symbol(INTEGER_LITERAL, Integer.valueOf((int) parseLong(0, yylength(), 8))); }
//  {OctLongLiteral}               { return symbol(INTEGER_LITERAL, new Long(parseLong(0, yylength()-1, 8))); }

  {BinaryIntegerLiteral}         { return symbol(INTEGER_LITERAL, Integer.valueOf((int) parseLong(2, yylength(), 2))); }
  //{BinaryLongLiteral}            { return symbol(INTEGER_LITERAL, new Long(parseLong(2, yylength()-1, 2))); }

  {FloatF32Literal}                 { return symbol(FLOATING_POINT_LITERAL, new Float(yytext().substring(0,yylength()-1))); }
  {FloatF64Literal}                { return symbol(FLOATING_POINT_LITERAL, new Double(yytext())); }
  //{DoubleLiteral}[dD]            { return symbol(FLOATING_POINT_LITERAL, new Double(yytext().substring(0,yylength()-1))); }

  /* comments */
  {Comment}                      { /* ignore */ }
  {EndOfLineComment}             {return symbol(LINE_TERMINATOR);}

  /* whitespace */
  {WhiteSpace}                   { /* if(includeWhiteSpace)return symbol(WHITESPACE); */ }

  /* identifiers */
  {Identifier}                   { return symbol(IDENTIFIER, yytext()); }
}

<STRING> {
  \"                             { yybegin(YYINITIAL); return symbol(STRING_LITERAL, yyline, columnStringStart, charStringStart, yycolumn - columnStringStart + 1 , string.toString()); }

  {StringCharacter}+             { string.append( yytext() ); }

  /* escape sequences */
  "\\b"                          { string.append( '\b' ); }
  "\\t"                          { string.append( '\t' ); }
  "\\n"                          { string.append( '\n' ); }
  "\\f"                          { string.append( '\f' ); }
  "\\r"                          { string.append( '\r' ); }
  "\\\""                         { string.append( '\"' ); }
  "\\'"                          { string.append( '\'' ); }
  "\\\\"                         { string.append( '\\' ); }
  \\[0-3]?{OctDigit}?{OctDigit}  { char val = (char) Integer.parseInt(yytext().substring(1),8);
                        				   string.append( val ); }

  /* error cases */
  \\.                            {yybegin(YYINITIAL); return symbol(error, yyline, columnStringStart, charStringStart, yycolumn - columnStringStart + 1 , "Illegal escape sequence \""+yytext()+"\""); }
  {LineTerminator}               {yybegin(YYINITIAL); return symbol(error, yyline, columnStringStart, charStringStart, yycolumn - columnStringStart + 1 , "Unterminated string at end of line"); }
}

<CHARLITERAL> {
  {SingleCharacter}\'            { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, yytext().charAt(0)); }

  /* escape sequences */
  "\\b"\'                        { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\b');}
  "\\t"\'                        { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\t');}
  "\\n"\'                        { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\n');}
  "\\f"\'                        { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\f');}
  "\\r"\'                        { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\r');}
  "\\\""\'                       { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\"');}
  "\\'"\'                        { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\'');}
  "\\\\"\'                       { yybegin(YYINITIAL); return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, '\\'); }
  \\[0-3]?{OctDigit}?{OctDigit}\' { yybegin(YYINITIAL);
			                              int val = Integer.parseInt(yytext().substring(1,yylength()-1),8);
			                            return symbol(CHARACTER_LITERAL, yyline, yycolumn - 1, yychar - 1, yylength() + 1, (char)val); }

  /* error cases */
  \\.                            {yybegin(YYINITIAL); return symbol(error, yyline, yycolumn - 1, yychar - 1, yylength() + 1,  "Illegal escape sequence \""+yytext()+"\""); }
  {LineTerminator}               {yybegin(YYINITIAL); return symbol(error, yyline, yycolumn - 1, yychar - 1, yylength() + 1, "Unterminated character literal at end of line"); }
}

/* error fallback */
[^]                              {yybegin(YYINITIAL); return symbol(error, yyline, yycolumn, yychar, yylength(), "Illegal character \""+yytext()+
                                                              "\" at line "+(yyline + 1)+", column "+(yycolumn + 1)); }
<<EOF>>                          { return symbol(EOF); }
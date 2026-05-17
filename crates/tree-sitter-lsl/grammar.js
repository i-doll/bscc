/**
 * @file LSL grammar for tree-sitter
 * @author Amalthea Skydancer <five@th3a.dev>
 * @license MIT OR Apache-2.0
 *
 * LSL operator precedence ranks (low → high) follow the Second Life wiki
 * (https://wiki.secondlife.com/wiki/LSL_Operators). Note that && and || share
 * one precedence level in LSL, unlike C.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  ASSIGN:  1,
  LOGICAL: 2,  // && and || share this level in LSL
  BIT_OR:  3,
  BIT_XOR: 4,
  BIT_AND: 5,
  EQ:      6,
  REL:     7,
  SHIFT:   8,
  ADD:     9,
  MUL:    10,
  CAST:   11,
  UNARY:  12,
  CALL:   13,
};

const BUILTIN_TYPES = [
  'integer', 'float', 'string', 'key',
  'vector', 'rotation', 'quaternion',
  'list',
];

module.exports = grammar({
  name: 'lsl',

  extras: $ => [
    /[\s\r\n]/,
    $.line_comment,
    $.block_comment,
    $.preproc_directive,
  ],

  word: $ => $.identifier,

  conflicts: _ => [],

  rules: {
    // ────────────────────────────── Top level ──────────────────────────────

    source_file: $ => repeat(choice(
      $.global_variable,
      $.function_declaration,
      $.default_state,
      $.state_declaration,
    )),

    global_variable: $ => seq(
      field('type', $.type),
      field('name', $.identifier),
      optional(seq('=', field('value', $._expression))),
      ';',
    ),

    function_declaration: $ => seq(
      optional(field('return_type', $.type)),
      field('name', $.identifier),
      '(',
      optional($.parameter_list),
      ')',
      field('body', $.block),
    ),

    parameter_list: $ => commaSep1($.parameter),
    parameter: $ => seq(field('type', $.type), field('name', $.identifier)),

    default_state: $ => seq(
      'default',
      '{',
      repeat($.event_handler),
      '}',
    ),

    state_declaration: $ => seq(
      'state',
      field('name', $.identifier),
      '{',
      repeat($.event_handler),
      '}',
    ),

    event_handler: $ => seq(
      field('name', $.identifier),
      '(',
      optional($.parameter_list),
      ')',
      field('body', $.block),
    ),

    type: _ => choice(...BUILTIN_TYPES),

    // ────────────────────────────── Statements ──────────────────────────────

    block: $ => seq('{', repeat($._statement), '}'),

    _statement: $ => choice(
      $.block,
      $.variable_declaration,
      $.expression_statement,
      $.if_statement,
      $.while_statement,
      $.do_statement,
      $.for_statement,
      $.return_statement,
      $.jump_statement,
      $.label_statement,
      $.state_change_statement,
      $.empty_statement,
    ),

    variable_declaration: $ => seq(
      field('type', $.type),
      field('name', $.identifier),
      optional(seq('=', field('value', $._expression))),
      ';',
    ),

    expression_statement: $ => seq($._expression, ';'),

    // `else` binds to the nearest unmatched `if`.
    if_statement: $ => prec.right(seq(
      'if', '(', field('condition', $._expression), ')',
      field('consequence', $._statement),
      optional(seq('else', field('alternative', $._statement))),
    )),

    while_statement: $ => seq(
      'while', '(', field('condition', $._expression), ')',
      field('body', $._statement),
    ),

    do_statement: $ => seq(
      'do', field('body', $._statement),
      'while', '(', field('condition', $._expression), ')', ';',
    ),

    for_statement: $ => seq(
      'for', '(',
      field('init', optional($._expression)), ';',
      field('condition', optional($._expression)), ';',
      field('update', optional($._expression)), ')',
      field('body', $._statement),
    ),

    return_statement: $ => seq('return', optional($._expression), ';'),

    jump_statement: $ => seq('jump', field('label', $.identifier), ';'),

    label_statement: $ => seq('@', field('name', $.identifier), ';'),

    state_change_statement: $ => seq(
      'state', field('name', $.identifier), ';',
    ),

    empty_statement: _ => ';',

    // ────────────────────────────── Expressions ──────────────────────────────

    _expression: $ => choice(
      $.assignment_expression,
      $.binary_expression,
      $.unary_expression,
      $.update_expression,
      $.cast_expression,
      $.call_expression,
      $.member_expression,
      $.parenthesized_expression,
      $.list_literal,
      $.vector_literal,
      $.rotation_literal,
      $.identifier,
      $.integer_literal,
      $.float_literal,
      $.string_literal,
    ),

    assignment_expression: $ => prec.right(PREC.ASSIGN, seq(
      field('left', $._expression),
      field('operator', choice('=', '+=', '-=', '*=', '/=', '%=')),
      field('right', $._expression),
    )),

    binary_expression: $ => {
      const table = [
        [PREC.LOGICAL, choice('&&', '||')],
        [PREC.BIT_OR,  '|'],
        [PREC.BIT_XOR, '^'],
        [PREC.BIT_AND, '&'],
        [PREC.EQ,      choice('==', '!=')],
        [PREC.REL,     choice('<', '<=', '>', '>=')],
        [PREC.SHIFT,   choice('<<', '>>')],
        [PREC.ADD,     choice('+', '-')],
        [PREC.MUL,     choice('*', '/', '%')],
      ];
      return choice(...table.map(([p, op]) =>
        prec.left(p, seq(
          field('left', $._expression),
          field('operator', op),
          field('right', $._expression),
        )),
      ));
    },

    unary_expression: $ => prec.right(PREC.UNARY, seq(
      field('operator', choice('!', '~', '-', '+')),
      field('argument', $._expression),
    )),

    update_expression: $ => prec.right(PREC.UNARY, choice(
      seq(field('operator', choice('++', '--')), field('argument', $._expression)),
      seq(field('argument', $._expression), field('operator', choice('++', '--'))),
    )),

    // `(integer) x` etc. Tight binding so `(integer) -1` is unary on the
    // cast, not subtraction.
    cast_expression: $ => prec.right(PREC.CAST, seq(
      '(', field('type', $.type), ')',
      field('value', $._expression),
    )),

    call_expression: $ => prec(PREC.CALL, seq(
      field('function', $.identifier),
      '(',
      optional($.argument_list),
      ')',
    )),

    argument_list: $ => commaSep1($._expression),

    member_expression: $ => prec.left(PREC.CALL, seq(
      field('object', $._expression),
      '.',
      field('property', $.identifier),
    )),

    parenthesized_expression: $ => seq('(', $._expression, ')'),

    list_literal: $ => seq('[', optional(commaSep1($._expression)), ']'),

    // Vector and rotation literals: `<x, y, z>` and `<x, y, z, s>`. Dynamic
    // precedence biases parsing toward literals over a chain of < comparisons
    // when the comma-and-arity shape fits. L2 widens corpus coverage of the
    // genuinely ambiguous cases (e.g. `<a < b, c, d>`).
    vector_literal: $ => prec.dynamic(1, seq(
      '<',
      field('x', $._expression), ',',
      field('y', $._expression), ',',
      field('z', $._expression),
      '>',
    )),

    rotation_literal: $ => prec.dynamic(1, seq(
      '<',
      field('x', $._expression), ',',
      field('y', $._expression), ',',
      field('z', $._expression), ',',
      field('s', $._expression),
      '>',
    )),

    // ────────────────────────────── Literals ──────────────────────────────

    // Integers: decimal or hex. No leading minus — that's handled by unary.
    integer_literal: _ => token(choice(
      /\d+/,
      /0[xX][0-9a-fA-F]+/,
    )),

    // Floats need a `.` or exponent to differentiate from integers.
    float_literal: _ => token(choice(
      /\d+\.\d*([eE][-+]?\d+)?[fF]?/,
      /\.\d+([eE][-+]?\d+)?[fF]?/,
      /\d+[eE][-+]?\d+[fF]?/,
    )),

    string_literal: _ => token(seq(
      '"',
      repeat(choice(
        /[^"\\\n]/,
        /\\./,
      )),
      '"',
    )),

    identifier: _ => /[A-Za-z_][A-Za-z0-9_]*/,

    // ────────────────────────────── Comments ──────────────────────────────

    line_comment: _ => token(seq('//', /[^\n]*/)),

    block_comment: _ => token(seq(
      '/*',
      /[^*]*\*+([^/*][^*]*\*+)*/,
      '/',
    )),

    // Firestorm / OSSL preprocessor extension. Parsed as an opaque token from
    // `#` to end of line, with `\`-newline allowed as a line continuation so
    // long #define bodies don't break parsing. Recognized directives in the
    // ecosystem: include, define, undef, if, ifdef, ifndef, elif, else,
    // endif, pragma, error, warning. We don't structure them further; tools
    // can re-parse the captured text if they want more detail.
    preproc_directive: _ => token(seq(
      '#',
      repeat(choice(
        /[^\n\\]/,
        seq('\\', /[\r\n]/),
      )),
    )),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

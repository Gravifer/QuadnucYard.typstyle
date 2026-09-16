/// typstyle: wrap_text=fill-sentence skip_consistency

这是第一句话。这是第二句话。这是第三句话。

#let value = [value]
这是 mixed Latin sentence。Next English sentence follows. 这是 #value 后面的句子。
版本 1.2.3 保持在同一句。Dr. Wang 后面继续一句英文。Another Latin sentence with #value.

这是一句故意很长的中文句子并混合 Latin words plus #value and #(1 + 2) so that the narrow snapshot widths must wrap this single sentence across multiple output lines。短句继续。

A Latin sentence that was already broken
across two source lines should be joined。これは最初の文です。これは2番目の文です。

中文和 *强调文本* 在一起。这句包含 `code`。这句包含 $x + y$。

#[
  外层内容块第一句。外层内容块第二句包含 #value。

  #[
    嵌套内容第一句。Nested Latin sentence that is long enough to require wrapping at the narrow widths with #(1 + 2)。最后一句中文。
  ]
]

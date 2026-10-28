package io.github.h0ffmann.ptbr_writer

import org.json.JSONObject

// The Rust side of the overlay (app/src-tauri/src/android.rs, overlay.rs), in the app's own
// libptbr_app.so. Each call returns `{"view": View}` or `{"error": message}` (app/wire's View).
object Ptbr {
    init {
        System.loadLibrary("ptbr_app")
    }

    // Blocks on the request to Claude: never on the main thread.
    external fun check(dataDir: String, text: String): String
    external fun accept(id: Int): String
    external fun ignore(id: Int): String
    external fun clear()
}

data class Card(
    val id: Int,
    val rule: String,
    val title: String,
    val severity: String,
    val original: String,
    val replacement: String,
    val detectOnly: Boolean,
)

// An underline: the card's words, as UTF-16 offsets into the text (a Kotlin String's indices).
data class Mark(val start: Int, val end: Int, val card: Int)

class Checked(val text: String, val cards: List<Card>, val marks: List<Mark>)

sealed class Reply {
    class Ok(val checked: Checked) : Reply()
    class Err(val message: String) : Reply()
}

fun parseReply(json: String): Reply {
    val o = JSONObject(json)
    if (o.has("error")) return Reply.Err(o.getString("error"))
    val v = o.getJSONObject("view")
    val cards = v.getJSONArray("cards").let { a ->
        (0 until a.length()).map { i ->
            val c = a.getJSONObject(i)
            Card(
                c.getInt("id"), c.getString("rule"), c.getString("title"), c.getString("severity"),
                c.getString("original"), c.getString("replacement"), c.getBoolean("detect_only"),
            )
        }
    }
    // The segments cover the text in order: their lengths are the offsets.
    val marks = mutableListOf<Mark>()
    var at = 0
    v.getJSONArray("segments").let { a ->
        for (i in 0 until a.length()) {
            val s = a.getJSONObject(i)
            val len = s.getString("text").length
            if (!s.isNull("card")) marks.add(Mark(at, at + len, s.getInt("card")))
            at += len
        }
    }
    return Reply.Ok(Checked(v.getString("text"), cards, marks))
}

// sample.lsl — bscc integration fixture for the tree-sitter LSL plugin.
// Designed to exercise functions, events, branches, and literals.

integer channel = 0;
string label = "click me";
vector home = <0.0, 0.0, 0.0>;

// TODO: localize this greeting
greet(string name) {
    llSay(channel, "hi " + name);
}

integer classify(integer n) {
    if (n < 0) {
        return -1;
    }
    if (n == 0) {
        return 0;
    }
    if (n > 100) {
        return 2;
    }
    return 1;
}

default {
    state_entry() {
        llListen(channel, "", NULL_KEY, "");
        llSetText(label, <1.0, 1.0, 1.0>, 1.0);
    }

    touch_start(integer total_number) {
        integer i;
        for (i = 0; i < total_number; ++i) {
            key toucher = llDetectedKey(i);
            if (toucher != NULL_KEY) {
                greet(llKey2Name(toucher));
            }
        }
    }

    listen(integer chan, string name, key id, string msg) {
        if (msg == "stop") {
            state idle;
        } else {
            llSay(0, "got: " + msg);
        }
    }
}

state idle {
    state_entry() {
        llSay(0, "going idle");
        state default;
    }
}

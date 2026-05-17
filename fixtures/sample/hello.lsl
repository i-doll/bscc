// Hello-world LSL fixture for bscc.
default
{
    state_entry()
    {
        llSay(0, "Hello, Avatar!");
    }

    touch_start(integer total_number)
    {
        /* Called when an avatar
           touches the object. */
        llSay(0, "Touched.");
    }
}

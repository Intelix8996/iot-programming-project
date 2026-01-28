"use client";

import { useEffect, useState } from "react";
import PinState from "./PinState";
import React from "preact/compat";

type PinProps = {
    gpio: string
}

export default function OutputPin({ gpio }: PinProps) {
    const [pinState, setPinState] = useState(true);

    useEffect(() => {
        readPin(gpio)
    }, []);

    function readPin(pin: string) {
        fetch("/" + pin + "/level")
        .then((resp) => {
            if (resp.ok)
                return resp.json()
            else
                console.error("Can't toggle pin")
        })
        .then((json) => {
            setPinState(json.level)
        })
    }

    function setPin(pin: string) {
        fetch("/" + pin + "/set/high").then((resp) => {
            if (resp.ok)
                setPinState(true)
            else
                console.error("Can't toggle pin")
        })
    }

    function resetPin(pin: string) {
        fetch("/" + pin + "/set/low").then((resp) => {
            if (resp.ok)
                setPinState(false)
            else
                console.error("Can't toggle pin")
        })
    }

    function togglePin(pin: string) {
        fetch("/" + pin + "/toggle").then((resp) => {
            if (resp.ok)
                setPinState(!pinState)
            else
                console.error("Can't toggle pin")
        })
    }

    return (
        <div className="flex flex-row m-2">
            <PinState state={pinState}/>
            <button className="btn" onClick={() => setPin(gpio)}>High</button>
            <button className="btn" onClick={() => resetPin(gpio)}>Low</button>
            <button className="btn" onClick={() => togglePin(gpio)}>Toggle</button>
        </div>
    );
}

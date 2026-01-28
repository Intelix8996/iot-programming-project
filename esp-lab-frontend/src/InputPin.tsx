import { useEffect, useState } from "preact/hooks";
import PinState from "./PinState";
import React from "preact/compat";

type PinProps = {
    gpio: string
}

export default function InputPin({ gpio }: PinProps) {
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

    return (
        <div className="flex flex-row m-2">
            <PinState state={pinState}/>
            <button className="btn" onClick={() => readPin(gpio)}>Read</button>
        </div>
    );
}

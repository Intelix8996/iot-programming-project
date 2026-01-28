"use client";

import { useEffect, useState } from "preact/hooks";
import InputPin from "./InputPin";
import OutputPin from "./OutputPin";
import React from "preact/compat";

export default function PinContainer({ gpio }) {
    const [mode, setMode] = useState("input");

    useEffect(() => {
        readMode(0)
    }, []);

    function readMode(retry: number) {
        fetch("/" + gpio + "/mode").then((resp) => {
            if (resp.ok)
                return resp.json()
            else
                if (retry < 2) {
                    setTimeout(() => {
                        readMode(retry + 1)
                    }, 2000);
                }
                console.error("Can't toggle pin")
        })
        .then((json) => {
            if (json) {
                console.log("Got mode for " + gpio + " " + json.mode)
                setMode(json.mode)
            }
        })
    }

    let pinBody;

    if (mode === "input")
        pinBody = <InputPin gpio={gpio} />
    else
        pinBody = <OutputPin gpio={gpio} />

    function setOutput(pin: string) {
        fetch("/" + pin + "/mode/output").then((resp) => {
            if (resp.ok)
                setMode("output")
            else
                console.error("Can't toggle pin")
        })
    }

    function setInput(pin: string) {
        fetch("/" + pin + "/mode/input").then((resp) => {
            if (resp.ok)
                setMode("input")
            else
                console.error("Can't toggle pin")
        })
    }

    function onModeChange(newMode: string) {
        if (newMode === "input")
            setInput(gpio)
        else
            setOutput(gpio)
    }

    return (
        <div className="flex flex-row items-center gap-x-5 bg-gray-200 rounded-box">
            <a className="m-5 p-1 rounded-box bg-gray-300">{gpio}</a>

            <label className="select">
                <span className="label">Mode</span>
                <select
                    value={mode}
                    onChange={(event) => onModeChange(event.target.value)}
                >
                    <option>input</option>
                    <option>output</option>
                </select>
            </label>

            { pinBody }
        </div >
    );
}

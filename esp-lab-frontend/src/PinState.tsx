"use client";

type PinStateProps = {
    state: boolean
}

export default function PinState({ state }: PinStateProps) {
    let icon;

    if (state)
        icon = <div className="bg-green-600 size-5 rounded-4xl justify-center m-2"></div>
    else
        icon = <div className="bg-red-600 size-5 rounded-4xl justify-center m-2"></div>

    return (
        <div className="bg-gray-300 rounded-box m-2">
            {icon}
        </div>
    );
}

import PinContainer from "./PinContainer";

import useGpios from "./api";

export function App() {
  const { gpios, isLoading, isError } = useGpios();

  let content;
  if (isLoading)
    content = <span className="loading loading-infinity loading-md"></span>;
  else if (isError)
    content = <h1>Can't load pins</h1>
  else {
    const pins = gpios.gpios;
    const pinContainers = pins.map((pin) => <li key={pin + "_li"} className="list-col ml-5 mr-5"><PinContainer key={pin} gpio={pin} /></li>)

    content = (
      <ul className="list bg-base-100 rounded-box shadow-md w-auto gap-2 p-5">
        <li className="p-4 pb-2 text-xs opacity-60 tracking-wide">GPIO Pins</li>
        {pinContainers}
      </ul>
    )
  }

    // const pins = ["gpio8", "gpio21"];
    // const pinContainers = pins.map((pin) => <li key={pin + "_li"} className="list-col ml-5 mr-5"><PinContainer key={pin} gpio={pin} /></li>)

    // let content = (
    //   <ul className="list bg-base-100 rounded-box shadow-md w-auto gap-2 p-5">
    //     <li className="p-4 pb-2 text-xs opacity-60 tracking-wide">GPIO Pins</li>
    //     {pinContainers}
    //   </ul> );

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black">
      <main className="flex min-h-screen w-full max-w-3xl flex-col items-center justify-between py-32 px-16 bg-white dark:bg-black sm:items-start">

        {content}

      </main>
    </div>
  );
}

import useSWR from "swr";

const fetcher = (...args) => fetch(...args).then(res => res.json())

export default function useGpios() {
    const { data, error, isLoading } = useSWR("/gpios", fetcher)
    return {
        gpios: data,
        isLoading,
        isError: error
    }
}

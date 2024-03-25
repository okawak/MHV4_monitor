import ShowDate from "@/components/show-date";
import PrintButton from "@/components/print-button";

export default function Home() {
    return (
        <main>
            <h1 className="text-3xl font-bold bg-gray-100 px-5 py-5">MHV4 monitor</h1>
            <ShowDate />
            <PrintButton />
        </main>
    );
}

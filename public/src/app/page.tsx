import ShowDate from "@/components/show-date";
import PrintButton from "@/components/print-button";
import NumberArrayComponent from "@/components/mhv4-table";

export default function Home() {
  return (
    <main>
      <h1 className="bg-gray-100 px-5 py-5 text-3xl font-bold">MHV4 monitor</h1>
      <ShowDate />
      <PrintButton />
      <NumberArrayComponent />
    </main>
  );
}

"use client";

function PrintButton() {
  const printWindow = () => {
    window.print();
  };

  let style_str = "inline-flex items-center justify-center";
  style_str += " whitespace-nowrap rounded-md text-sm font-medium";
  style_str += " ring-offset-background transition-colors";
  style_str += " focus-visible:outline-none focus-visible:ring-2";
  style_str += " focus-visible:ring-ring focus-visible:ring-offset-2";
  style_str += " disabled:pointer-events-none disabled:opacity-50";

  style_str += " bg-green-700 text-primary-foreground hover:bg-green-700/80";
  style_str += " mx-1 h-8 px-4 py-2";

  return (
    <div className={style_str}>
      <button onClick={printWindow}>print</button>
    </div>
  );
}

export default PrintButton;

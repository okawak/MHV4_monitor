"use client";

import React, { useState, useEffect } from "react";

import { useMHV4Data } from "@/contexts/MHV4Context";
import ShowDate from "@/components/show-date";
import RCButton from "@/components/rc-button";
import PrintButton from "@/components/print-button";
import MHV4Table from "@/components/mhv4-table";
import ApplyButton from "@/components/apply-button";
//import OnoffButton from "@/components/onoff-button";

export default function Home() {
  const { rcType, voltageArray } = useMHV4Data();
  const [inputValues, setInputValues] = useState<number[]>([]);

  useEffect(() => {
    setInputValues(new Array(voltageArray.length).fill(0));
  }, [voltageArray.length]);

  const handleValueChange = (newValue: number, index: number) => {
    const updatedValues = inputValues.map((value, i) =>
      i === index ? newValue : value,
    );
    setInputValues(updatedValues);
  };

  return (
    <main>
      <h1 className="bg-gray-100 px-5 py-5 text-3xl font-bold">MHV4 monitor</h1>
      <ShowDate />
      <PrintButton />
      <RCButton do_rc={rcType} />
      <MHV4Table
        onValueChange={(newValue, index) => handleValueChange(newValue, index)}
      />
      <ApplyButton inputs={inputValues} />
    </main>
  );
}

import{r as h}from"./index.NEDEFKed.js";var m={exports:{}},s={};/**
 * @license React
 * react-jsx-runtime.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var p=h,b=Symbol.for("react.element"),v=Symbol.for("react.fragment"),g=Object.prototype.hasOwnProperty,_=p.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED.ReactCurrentOwner,w={key:!0,ref:!0,__self:!0,__source:!0};function f(e,r,o){var t,a={},d=null,c=null;o!==void 0&&(d=""+o),r.key!==void 0&&(d=""+r.key),r.ref!==void 0&&(c=r.ref);for(t in r)g.call(r,t)&&!w.hasOwnProperty(t)&&(a[t]=r[t]);if(e&&e.defaultProps)for(t in r=e.defaultProps,r)a[t]===void 0&&(a[t]=r[t]);return{$$typeof:b,type:e,key:d,ref:c,props:a,_owner:_.current}}s.Fragment=v;s.jsx=f;s.jsxs=f;m.exports=s;var n=m.exports;function j({title:e,children:r,className:o}){return n.jsxs("div",{className:`
    bg-base-background-color
    ring-box-color-standard
    ring-4
    break-words
    overflow-hidden
    shadow-square
    shadow-box-color-standard/50
    min-h-[120px]
    min-w-[120px]
    flex flex-col
    m-1
    px-3
    pb-3
    sm:max-h-notfullscreen
    text-base-text-color
    ${o}
    `,children:[n.jsx("div",{className:`\r
            bg-box-color-standard\r
            mx-[-12px]\r
            min-w-max\r
            min-h-min\r
            items-end\r
            text-left\r
            text-end\r
            flex\r
            justify-end\r
            align-middle\r
            p-1\r
            pb-2\r
            gap-1\r
            text-base-background-color\r
        `,children:"➖⏹️❌"}),n.jsxs("div",{className:`\r
            sm:overflow-y-auto\r
            scrollbar-thin\r
            scrollbar-track-base-background-color\r
            scrollbar-thumb-box-color-standard`,children:[n.jsx("div",{className:`\r
            font-headerf\r
            text-2xl\r
            md:text-3xl\r
            lg:text-4xl\r
        `,children:e}),n.jsx("div",{children:r})]})]})}var l=(e=>(e[e.standard=0]="standard",e[e.werwolvdark=1]="werwolvdark",e[e.trans=2]="trans",e[e.nostalgia=3]="nostalgia",e))(l||{});function k(){let e=i();return x(),n.jsx("div",{children:n.jsx("a",{href:"#",onClick:r=>{r.preventDefault(),e=i();const o=e<u?e+1:0;localStorage.setItem("theme_v2",o.toString()),x();const t=document.querySelector(".theme-name");t&&(t.textContent=l[o])},children:n.jsx(j,{title:"",children:n.jsxs("div",{className:"content-center p-3 text-center",children:[n.jsx("div",{className:`
                  font-headerf
                  text-lg
                  md:text-xl
                  lg:text-2xl
                  bg-base-text-color/25
                  rounded-lg
                  p-3
                `,children:"Change Theme"}),n.jsx("div",{className:"theme-name font-light text-base-text-color/40 mb-[-12px]",children:`${l[e]}`})]})})})})}const u=3;function x(){const e=i();document.documentElement.classList.add(l[e]);for(let r=0;r<=u;r++)l[r]!==l[e]&&document.documentElement.classList.remove(l[r])}function i(){let e=null,r=0;return typeof window<"u"&&(e=localStorage.getItem("theme_v2")),typeof e=="number"?r=e:typeof e=="string"?r=parseInt(e):r=0,r}export{k as T,x as l};

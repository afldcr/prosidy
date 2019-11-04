<?xml version="1.0" encoding="utf-8"?>
<!--
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
-->
<xsl:transform version="1.0" xmlns:exsl="http://exslt.org/common" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="https://prosidy.org/schema/prosidy.xsd" xmlns:pm="https://prosidy.org/schema/prosidy-manual.xsd" exclude-result-prefixes="exsl p pm">
    <xsl:output method="html" indent="no" omit-xml-declaration="yes" />

    <xsl:strip-space elements="*" />

    <xsl:template match="/p:document">
        <html lang="{./@lang}">
            <head>
                <title><xsl:value-of select="./@title" /></title>
            </head>
            <body>
                <header>
                    <h1><xsl:value-of select="./@title" /></h1>
                </header>
                <main><xsl:apply-templates /></main>
            </body>
        </html>
    </xsl:template>

    <xsl:template match="/p:document/p:paragraph">
        <p><xsl:apply-templates /></p>
    </xsl:template>

    <xsl:template match="p:paragraph">
        <xsl:apply-templates />
    </xsl:template>

    <xsl:template match="pm:b">
        <strong><xsl:apply-templates /></strong>
    </xsl:template>

    <xsl:template match="pm:i">
        <em><xsl:apply-templates /></em>
    </xsl:template>

    <xsl:template match="pm:prosidy">
        <xsl:text>Prosidy</xsl:text>
    </xsl:template>

    <xsl:template match="pm:link">
        <xsl:call-template name="hyperlink">
            <xsl:with-param name="key" select="@pm:to" />
        </xsl:call-template>
    </xsl:template>

    <xsl:template match="pm:example">
        <pre>
            <code>
                <xsl:apply-templates />
            </code>
        </pre>
    </xsl:template>

    <xsl:template match="pm:section">
        <xsl:element name="h{@pm:depth + 1}">
            <xsl:attribute name="id">
                <xsl:value-of select="@pm:id" />
            </xsl:attribute>
            <xsl:apply-templates />
        </xsl:element>
    </xsl:template>

    <xsl:template match="pm:toc">
        <nav>
            <h2>Contents</h2>
            <xsl:call-template name="make-toc" />
        </nav>
    </xsl:template>

    <xsl:template match="p:literal">
        <xsl:value-of select="." />
    </xsl:template>

    <xsl:template match="pm:lit">
        <code><xsl:apply-templates /></code>
    </xsl:template>

    <xsl:template match="*">
        <xsl:message terminate="yes">
            <xsl:text>Untemplated node: </xsl:text>
            <xsl:call-template name="local-path" />
        </xsl:message>
    </xsl:template>

    <!--
        A fancy lil table of contents generator
    -->

    <xsl:key
        name="subsections"
        match="//pm:section"
        use="generate-id(preceding-sibling::*[@pm:depth = current()/@pm:depth - 1])" />

    <xsl:template name="make-toc">
        <xsl:param name="sections" select="exsl:node-set(//pm:section[@pm:depth=1])" />
        <xsl:if test="count($sections) > 0">
            <ol>
                <xsl:for-each select="$sections">
                    <li>
                        <div><a href="#{@pm:id}"><xsl:apply-templates /></a></div>
                        <xsl:call-template name="make-toc">
                            <xsl:with-param name="sections" select="key('subsections', generate-id(.))" />
                        </xsl:call-template>
                    </li>
                </xsl:for-each>
            </ol>
        </xsl:if>
    </xsl:template>

    <!--
        Maintaining an external links table.
    -->

    <xsl:variable name="hyperlinks" select="document('hyperlinks.xml')/hyperlinks/link" />

    <xsl:template name="hyperlink" mode="function">
        <xsl:param name="key">
            <xsl:message terminate="yes">Parameter 'key' not provided to template 'hyperlink'</xsl:message>
        </xsl:param>
        <xsl:variable name="link" select="$hyperlinks[@key=$key]" />
        <xsl:if test="count($link)=0">
            <xsl:message terminate="yes">
                <xsl:text>Unknown hyperlink target '</xsl:text>
                <xsl:value-of select="$key" />
                <xsl:text>'. Valid targets: </xsl:text>
                <xsl:for-each select="$hyperlinks">
                    <xsl:if test="position() > 1">
                        <xsl:text>, </xsl:text>
                    </xsl:if>
                    <xsl:text>'</xsl:text>
                    <xsl:value-of select="@key" />
                    <xsl:text>'</xsl:text>
                </xsl:for-each>
            </xsl:message>
        </xsl:if>
        <xsl:element name="a">
            <xsl:attribute name="href">
                <xsl:value-of select="$link/@url" />
            </xsl:attribute>
            <xsl:if test="contains($link/@url, '://')">
                <xsl:attribute name="target">_blank</xsl:attribute>
            </xsl:if>
            <xsl:choose>
                <xsl:when test="count(descendant::*)=0">
                    <xsl:for-each select="$link"><xsl:apply-templates /></xsl:for-each>
                </xsl:when>
                <xsl:otherwise>
                    <xsl:apply-templates />
                </xsl:otherwise>
            </xsl:choose>
        </xsl:element>
    </xsl:template>

    <!--
        Generating the path to the current node.
    -->

    <xsl:template name="local-path">
        <xsl:for-each select="ancestor::node() | .">
            <xsl:text>/</xsl:text>
            <xsl:value-of select="local-name()" />
        </xsl:for-each>
    </xsl:template>
</xsl:transform>
